<!--
  Presupuestos: cartas presupuesto de reparaciones (correctivo) y pequeñas obras.

  Flujo: nuevo presupuesto → explicar qué se ha roto o desgastado y qué se propone → líneas
  (repuesto, mano de obra, desplazamiento) → Guardar (se asigna el número P-AAAA-NNN) → Carta PDF.

  Los precios por defecto de mano de obra y desplazamiento salen de «Parámetros por año»; el del
  material se calcula desde su coste con el recargo del año, y se puede cambiar a mano.
  El coste y el margen son internos: no salen en la carta.
-->
<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import {
  api, eur, num,
  type LineaPresupuesto, type ModeloPresupuesto, type PrecioDefecto, type Presupuesto, type ResumenPresupuesto, type TotalesPresupuesto,
} from "../api";

const emit = defineEmits<{ cambio: [] }>();

const ESTADOS: Record<string, string> = { borrador: "Borrador", enviado: "Enviado", aceptado: "Aceptado", rechazado: "Rechazado" };
const AMBITOS: Record<string, string> = { "": "General", RITE: "RITE", PCI: "PCI", REBT: "Baja tensión" };
const TIPOS_LINEA: Record<string, string> = {
  material: "Repuesto / material", hora_oficial: "Hora oficial", hora_pareja: "Hora oficial + ayudante",
  salida: "Desplazamiento", partida: "Otra partida",
};

const lista = ref<ResumenPresupuesto[]>([]);
const filtro = ref("");
const filtroEstado = ref("");
const filtroCliente = ref("");
const filtroAmbito = ref("todos");
const modo = ref<"listado" | "editor">("listado");
const modelos = ref<ModeloPresupuesto[]>([]);
const verModelos = ref(false);
const filtroModelo = ref("");
const nuevoModelo = ref<{ codigo: string; grupo: string; nombre: string } | null>(null);
const contratos = ref<string[]>([]);
const p = ref<Presupuesto | null>(null);
const totales = ref<TotalesPresupuesto | null>(null);
const precios = ref<PrecioDefecto[]>([]);
const recargo = ref<number | null>(null);
const error = ref("");
const mensaje = ref("");

const hoy = () => new Date().toISOString().slice(0, 10);
const anio = computed(() => Number(p.value?.fecha.slice(0, 4)) || new Date().getFullYear());

const clientes = computed(() => [...new Set(lista.value.map((r) => r.cliente))].sort((a, b) => a.localeCompare(b, "es")));

const listaFiltrada = computed(() => {
  const f = filtro.value.trim().toLowerCase();
  return lista.value.filter((r) =>
    (!filtroEstado.value || r.estado === filtroEstado.value) &&
    (!filtroCliente.value || r.cliente === filtroCliente.value) &&
    (filtroAmbito.value === "todos" || r.ambito === filtroAmbito.value) &&
    (!f || `${r.numero} ${r.cliente} ${r.instalacion} ${r.equipo} ${r.titulo}`.toLowerCase().includes(f)));
});

/** Listado agrupado: cliente → instalación → presupuestos. */
const grupos = computed(() => {
  const g = new Map<string, Map<string, ResumenPresupuesto[]>>();
  for (const r of listaFiltrada.value) {
    const ins = r.instalacion || "Sin instalación indicada";
    if (!g.has(r.cliente)) g.set(r.cliente, new Map());
    const m = g.get(r.cliente)!;
    if (!m.has(ins)) m.set(ins, []);
    m.get(ins)!.push(r);
  }
  return [...g].map(([cliente, m]) => ({
    cliente,
    total: [...m.values()].flat().reduce((a, r) => a + r.base, 0),
    instalaciones: [...m].map(([nombre, filas]) => ({ nombre, filas })),
  }));
});

/** Resumen por estado de lo que se está mostrando (importes sin IVA). */
const resumen = computed(() => Object.keys(ESTADOS).map((k) => {
  const f = listaFiltrada.value.filter((r) => r.estado === k);
  return { estado: k, n: f.length, base: f.reduce((a, r) => a + r.base, 0) };
}));

/** Otros presupuestos del mismo cliente, para ver qué se le está ofertando. */
const delCliente = computed(() =>
  !p.value?.cliente ? [] : lista.value.filter((r) => r.cliente === p.value!.cliente && r.numero !== p.value!.numero));

async function cargarLista() { lista.value = await api.listarPresupuestos(); }
async function cargarPrecios() {
  const r = await api.preciosPresupuesto(anio.value);
  precios.value = r.filas; recargo.value = r.recargo_material_pct;
}

async function cargarModelos() { modelos.value = await api.listarModelos(); }

onMounted(async () => {
  await cargarLista();
  await cargarModelos();
  contratos.value = (await api.listarContratos()).map((c) => c.nombre);
});

async function intentar(f: () => Promise<void>) {
  error.value = ""; mensaje.value = "";
  try { await f(); } catch (e) { error.value = String(e); }
}

const abrir = (numero: string) => intentar(async () => {
  p.value = await api.cargarPresupuesto(numero);
  modo.value = "editor";
  p.value.lineas.forEach((l) => (l.auto = false));
  await cargarPrecios();
});

const nuevo = (tipo: string) => intentar(async () => {
  p.value = {
    numero: "", fecha: hoy(), ambito: filtroAmbito.value === "todos" ? "" : filtroAmbito.value, tipo, cliente: "", cif: "", direccion: "", contacto: "", contrato: "",
    titulo: "", equipo: "", deteccion: "", averia: "", consecuencias: "", propuesta: "",
    lineas: [], descuento_pct: 0, iva_pct: 21, validez_dias: 30, condiciones: "", estado: "borrador", notas_internas: "",
  };
  // Si se está viendo un cliente concreto, el presupuesto nuevo es para él.
  datosDelCliente();
  const previo = lista.value.find((r) => r.cliente === filtroCliente.value);
  if (previo && !p.value.ambito) p.value.ambito = previo.ambito;
  modo.value = "editor";
  await cargarPrecios();
  anadir("material");
  anadir(tipo === "correctivo" ? "hora_oficial" : "hora_pareja");
  anadir("salida");
});

/** Datos del cliente que se está viendo en el listado, para un presupuesto nuevo. */
function datosDelCliente() {
  if (!p.value || !filtroCliente.value) return;
  const previo = lista.value.find((r) => r.cliente === filtroCliente.value);
  p.value.cliente = filtroCliente.value;
  if (previo) { p.value.contrato = previo.contrato; p.value.direccion = previo.direccion; }
}

/** Modelos agrupados por grupo de equipos, filtrados por el buscador. */
const gruposModelos = computed(() => {
  const f = filtroModelo.value.trim().toLowerCase();
  const g = new Map<string, ModeloPresupuesto[]>();
  for (const m of modelos.value) {
    if (f && !`${m.grupo} ${m.nombre} ${m.codigo} ${m.titulo}`.toLowerCase().includes(f)) continue;
    if (!g.has(m.grupo)) g.set(m.grupo, []);
    g.get(m.grupo)!.push(m);
  }
  return [...g].map(([grupo, lista]) => ({ grupo, lista }));
});
const nombresGrupos = computed(() => [...new Set(modelos.value.map((m) => m.grupo))]);

const usarModelo = (codigo: string) => intentar(async () => {
  p.value = await api.presupuestoDesdeModelo(codigo, hoy());
  p.value.lineas.forEach((l) => (l.auto = l.tipo === "material"));
  datosDelCliente();
  await cargarPrecios();
  verModelos.value = false;
  modo.value = "editor";
});

const borrarModelo = (m: ModeloPresupuesto) => intentar(async () => {
  if (!confirm(`¿Borrar el modelo «${m.nombre}» (${m.codigo})?`)) return;
  await api.borrarModelo(m.codigo);
  await cargarModelos();
});

function prepararModelo() {
  if (!p.value) return;
  const desde = p.value.notas_internas.match(/modelo ([A-Z0-9-]+)/)?.[1];
  const base = modelos.value.find((m) => m.codigo === desde);
  nuevoModelo.value = { codigo: "", grupo: base?.grupo ?? "", nombre: p.value.titulo };
}

const guardarModelo = () => intentar(async () => {
  if (!p.value || !nuevoModelo.value) return;
  const { codigo, grupo, nombre } = nuevoModelo.value;
  if (!codigo.trim() || !nombre.trim() || !grupo.trim()) throw "Indique código, grupo y nombre del modelo.";
  if (modelos.value.some((m) => m.codigo === codigo.trim().toUpperCase()) && !confirm(`Ya existe el modelo ${codigo.toUpperCase()}. ¿Sustituirlo?`)) return;
  await api.guardarComoModelo(limpio(p.value), codigo, grupo, nombre);
  await cargarModelos();
  nuevoModelo.value = null;
  mensaje.value = `Modelo ${codigo.trim().toUpperCase()} guardado. Ya puede usarlo desde «+ Desde modelo».`;
});

/** Añade una línea con el precio y el coste por defecto de su tipo. */
function anadir(tipo: string) {
  const d = precios.value.find((x) => x.tipo === tipo);
  p.value?.lineas.push({
    tipo, descripcion: d?.descripcion ?? "", unidad: d?.unidad ?? "ud", cantidad: 1,
    precio_unitario: d?.precio_unitario ?? 0, coste_unitario: d?.coste_unitario ?? null, auto: tipo === "material",
  });
}

/** Material: el precio sigue al coste (con el recargo del año) hasta que se escribe a mano. */
function cambioCoste(l: LineaPresupuesto) {
  if (l.tipo === "material" && l.auto !== false && l.coste_unitario != null && recargo.value != null) {
    l.precio_unitario = Math.round(l.coste_unitario * (1 + recargo.value / 100) * 100) / 100;
  }
}

/** Al cambiar el tipo de una línea se proponen su unidad, precio y coste. */
function cambioTipo(l: LineaPresupuesto) {
  const d = precios.value.find((x) => x.tipo === l.tipo);
  if (!d) return;
  l.unidad = d.unidad;
  if (!l.descripcion) l.descripcion = d.descripcion;
  if (d.precio_unitario != null) l.precio_unitario = d.precio_unitario;
  if (d.coste_unitario != null) l.coste_unitario = d.coste_unitario;
  l.auto = l.tipo === "material";
}

/** Copia lista para el núcleo: los campos numéricos vacíos pasan a 0 o a «sin coste». */
function limpio(x: Presupuesto): Presupuesto {
  const n = (v: unknown) => (typeof v === "number" && isFinite(v) ? v : 0);
  return {
    ...x,
    descuento_pct: n(x.descuento_pct), iva_pct: n(x.iva_pct), validez_dias: Math.max(0, Math.round(n(x.validez_dias))),
    lineas: x.lineas.map(({ auto: _, ...l }) => ({
      ...l, cantidad: n(l.cantidad), precio_unitario: n(l.precio_unitario),
      coste_unitario: typeof l.coste_unitario === "number" && isFinite(l.coste_unitario) ? l.coste_unitario : null,
    })),
  };
}

// Totales: los calcula el núcleo en Rust cada vez que cambia algo.
let espera: number | undefined;
watch(p, () => {
  clearTimeout(espera);
  espera = window.setTimeout(async () => {
    if (p.value) totales.value = await api.calcularPresupuesto(limpio(p.value)).catch(() => null);
  }, 120);
}, { deep: true });
watch(anio, () => { if (p.value) cargarPrecios(); });

const guardar = () => intentar(async () => {
  if (!p.value) return;
  const numero = await api.guardarPresupuesto(limpio(p.value));
  p.value.numero = numero;
  mensaje.value = `Presupuesto ${numero} guardado.`;
  await cargarLista(); emit("cambio");
});

const pdf = () => intentar(async () => {
  if (!p.value) return;
  const numero = await api.guardarPresupuesto(limpio(p.value));
  p.value.numero = numero;
  await cargarLista();
  const ruta = await save({ defaultPath: `${numero}.pdf`, filters: [{ name: "PDF", extensions: ["pdf"] }] });
  if (!ruta) return;
  await api.pdfPresupuesto(numero, ruta);
  mensaje.value = `Carta presupuesto guardada en ${ruta}`;
});

const borrar = () => intentar(async () => {
  if (!p.value?.numero || !confirm(`¿Borrar el presupuesto ${p.value.numero}?`)) return;
  await api.borrarPresupuesto(p.value.numero);
  p.value = null; totales.value = null; modo.value = "listado";
  await cargarLista(); emit("cambio");
});

const volver = async () => { modo.value = "listado"; await cargarLista(); };

/** Relación en PDF de los presupuestos que se están mostrando (sin costes internos). */
const relacionPdf = () => intentar(async () => {
  const titulo = filtroCliente.value ? `Presupuestos · ${filtroCliente.value}` : "Relación de presupuestos";
  const ruta = await save({ defaultPath: `${titulo.replace(/[\\/:*?"<>|·]/g, "").trim()}.pdf`, filters: [{ name: "PDF", extensions: ["pdf"] }] });
  if (!ruta) return;
  await api.pdfRelacionPresupuestos(listaFiltrada.value.map((r) => r.numero), titulo, ruta);
  mensaje.value = `Relación guardada en ${ruta}`;
});

const fecha = (f: string) => f.split("-").reverse().join("/");
const TIPOS: Record<string, string> = { correctivo: "Reparación", obra: "Obra" };

/** Parámetros de precio que faltan para el año del presupuesto. */
const faltan = computed(() => {
  const n: string[] = precios.value
    .filter((d) => ["hora_oficial", "hora_pareja", "salida"].includes(d.tipo) && d.precio_unitario == null)
    .map((d) => TIPOS_LINEA[d.tipo]);
  if (recargo.value == null) n.push("recargo del material");
  return n;
});

const claseMargen = computed(() => !totales.value ? "" : totales.value.margen_pct < 15 ? "malo" : "bueno");
</script>

<template>
  <h1>Presupuestos</h1>
  <div v-if="error" class="error">{{ error }}</div>
  <div v-if="mensaje" class="aviso">{{ mensaje }}</div>

  <div class="fila cabecera">
    <button v-if="modo === 'editor'" @click="volver">← Listado</button>
    <span style="flex: 1"></span>
    <button class="primario" @click="verModelos = !verModelos">+ Desde modelo</button>
    <button @click="nuevo('correctivo')">+ Reparación en blanco</button>
    <button @click="nuevo('obra')">+ Obra / mejora</button>
  </div>

  <div v-if="verModelos" class="panel modelos">
    <div class="fila">
      <h2 style="margin: 0">Modelos de presupuesto</h2>
      <input v-model="filtroModelo" placeholder="Buscar: bomba, correas, detectores…" style="flex: 1" />
      <button @click="verModelos = false">Cerrar</button>
    </div>
    <p class="pequeño">Avería típica de cada equipo con su explicación, el repuesto y la mano de obra. Los precios se calculan con
      los parámetros del año. Para adaptar uno a otro equipo parecido: úselo, cámbielo y pulse «Guardar como modelo…».</p>
    <div v-for="g in gruposModelos" :key="g.grupo">
      <div class="instalacion">{{ g.grupo }}</div>
      <table class="listado">
        <tbody>
          <tr v-for="m in g.lista" :key="m.codigo" @click="usarModelo(m.codigo)">
            <td>{{ m.nombre }} <span v-if="m.ambito" class="ambito">{{ AMBITOS[m.ambito] ?? m.ambito }}</span>
              <span v-if="m.origen === 'propio'" class="ambito propio">propio</span>
              <div class="pequeño">{{ m.titulo }}</div></td>
            <td class="pequeño" style="width: 170px">{{ m.codigo }}</td>
            <td style="width: 160px" class="der">
              <button class="primario" @click.stop="usarModelo(m.codigo)">Usar</button>
              <button class="peligro" title="Borrar modelo" @click.stop="borrarModelo(m)">×</button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
    <div v-if="!gruposModelos.length" class="pequeño">Ningún modelo coincide.</div>
  </div>

  <template v-if="modo === 'listado'">
    <div class="panel">
      <div class="fila">
        <input v-model="filtro" placeholder="Buscar número, equipo, asunto…" style="flex: 1" />
        <select v-model="filtroCliente"><option value="">Todos los clientes</option><option v-for="c in clientes" :key="c" :value="c">{{ c }}</option></select>
        <select v-model="filtroAmbito"><option value="todos">Todos los ámbitos</option><option v-for="(t, k) in AMBITOS" :key="k" :value="k">{{ t }}</option></select>
        <select v-model="filtroEstado"><option value="">Todos los estados</option><option v-for="(t, k) in ESTADOS" :key="k" :value="k">{{ t }}</option></select>
        <button :disabled="!listaFiltrada.length" @click="relacionPdf">Relación PDF</button>
      </div>
      <div class="tarjetas" style="margin-top: 10px">
        <div v-for="r in resumen" :key="r.estado" :class="['tarjeta', 'est-' + r.estado]">
          <div class="etiqueta">{{ ESTADOS[r.estado] }} ({{ r.n }})</div>
          <div class="valor">{{ eur(r.base) }}</div>
        </div>
      </div>
      <div class="pequeño" style="margin-top: 4px">Importes sin IVA.</div>
    </div>

    <div v-if="!lista.length" class="panel pequeño" style="margin-top: 10px">
      Todavía no hay presupuestos. La carta explica al cliente qué se ha roto o desgastado, sus consecuencias y el
      repuesto que se propone, con el importe y un espacio para su aceptación.
    </div>
    <div v-else-if="!grupos.length" class="panel pequeño" style="margin-top: 10px">Ningún presupuesto cumple el filtro.</div>

    <div v-for="g in grupos" :key="g.cliente" class="panel" style="margin-top: 10px">
      <div class="fila entre"><h2 style="margin: 0">{{ g.cliente }}</h2><span class="pequeño">{{ eur(g.total) }} sin IVA</span></div>
      <template v-for="ins in g.instalaciones" :key="ins.nombre">
        <div class="instalacion">{{ ins.nombre }}</div>
        <table class="listado">
          <colgroup><col style="width: 105px" /><col style="width: 90px" /><col style="width: 90px" /><col /><col style="width: 105px" /><col style="width: 110px" /></colgroup>
          <tbody>
            <tr v-for="r in ins.filas" :key="r.numero" @click="abrir(r.numero)">
              <td><b>{{ r.numero }}</b></td>
              <td>{{ fecha(r.fecha) }}</td>
              <td class="pequeño">{{ TIPOS[r.tipo] ?? r.tipo }}<div v-if="r.ambito" class="ambito">{{ AMBITOS[r.ambito] ?? r.ambito }}</div></td>
              <td>{{ r.titulo }}<div v-if="r.equipo" class="pequeño">{{ r.equipo }}</div></td>
              <td><span :class="['estado', r.estado]">{{ ESTADOS[r.estado] }}</span></td>
              <td class="der">{{ eur(r.base) }}</td>
            </tr>
          </tbody>
        </table>
      </template>
    </div>
  </template>

  <div v-else-if="p">
      <div class="panel">
        <div class="fila">
          <label class="campo">Número<input :value="p.numero || 'al guardar'" disabled style="width: 120px" /></label>
          <label class="campo">Fecha<input type="date" v-model="p.fecha" /></label>
          <label class="campo">Tipo
            <select v-model="p.tipo"><option value="correctivo">Reparación</option><option value="obra">Obra / mejora</option></select>
          </label>
          <label class="campo" title="Decide el logo y el número de empresa habilitada de la carta">Ámbito (logo y habilitación)
            <select v-model="p.ambito"><option v-for="(t, k) in AMBITOS" :key="k" :value="k">{{ t }}</option></select>
          </label>
          <label class="campo">Estado
            <select v-model="p.estado"><option v-for="(t, k) in ESTADOS" :key="k" :value="k">{{ t }}</option></select>
          </label>
          <label class="campo" style="flex: 1">Contrato de mantenimiento (opcional)
            <input v-model="p.contrato" list="lista-contratos" />
            <datalist id="lista-contratos"><option v-for="c in contratos" :key="c" :value="c" /></datalist>
          </label>
        </div>
        <div class="fila" style="margin-top: 8px">
          <label class="campo" style="flex: 2">Cliente<input v-model="p.cliente" list="lista-clientes" /><datalist id="lista-clientes"><option v-for="c in clientes" :key="c" :value="c" /></datalist></label>
          <label class="campo">CIF/NIF<input v-model="p.cif" style="width: 120px" /></label>
          <label class="campo" style="flex: 1">A la atención de<input v-model="p.contacto" /></label>
        </div>
        <div class="fila" style="margin-top: 8px">
          <label class="campo" style="flex: 1">Instalación / dirección (si no hay contrato, agrupa el listado por este campo)<input v-model="p.direccion" /></label>
        </div>
      </div>

      <div class="panel" style="margin-top: 10px">
        <h2 style="margin-top: 0">Carta</h2>
        <div class="fila">
          <label class="campo" style="flex: 1">Asunto<input v-model="p.titulo" placeholder="Sustitución de…" /></label>
        </div>
        <div class="fila" style="margin-top: 8px">
          <label class="campo" style="flex: 1">Equipo afectado<input v-model="p.equipo" placeholder="la bomba circuladora B-1 del primario" /></label>
          <label v-if="p.tipo === 'correctivo'" class="campo" style="flex: 1">Detectado en<input v-model="p.deteccion" placeholder="revisión mensual del 14 de octubre de 2026" /></label>
        </div>
        <label class="campo bloque">{{ p.tipo === "correctivo" ? "Qué se ha roto o desgastado" : "Situación actual" }}
          <textarea v-model="p.averia" rows="3" />
        </label>
        <label class="campo bloque">Consecuencias de no actuar (opcional)
          <textarea v-model="p.consecuencias" rows="2" />
        </label>
        <label class="campo bloque">Solución propuesta (repuesto y trabajos)
          <textarea v-model="p.propuesta" rows="3" />
        </label>
      </div>

      <div class="panel" style="margin-top: 10px">
        <h2 style="margin-top: 0">Importe</h2>
        <div v-if="faltan.length" class="aviso">
          Faltan precios en «Parámetros por año» ({{ anio }}): {{ faltan.join(", ") }}. Mientras tanto esas líneas salen a 0 €;
          rellénelos allí una vez y se aplicarán a todos los presupuestos.
        </div>
        <table class="lineas">
          <colgroup>
            <col style="width: 170px" /><col /><col style="width: 80px" /><col style="width: 70px" />
            <col style="width: 100px" /><col style="width: 100px" /><col style="width: 100px" /><col style="width: 40px" />
          </colgroup>
          <thead><tr>
            <th>Tipo</th><th>Concepto</th><th class="der">Cant.</th><th>Ud.</th>
            <th class="der" title="Interno: no sale en la carta">Coste ud.</th><th class="der">Precio ud.</th><th class="der">Importe</th><th></th>
          </tr></thead>
          <tbody>
            <tr v-for="(l, i) in p.lineas" :key="i">
              <td><select v-model="l.tipo" @change="cambioTipo(l)"><option v-for="(t, k) in TIPOS_LINEA" :key="k" :value="k">{{ t }}</option></select></td>
              <td><input v-model="l.descripcion" /></td>
              <td class="der"><input class="num" type="number" step="any" v-model.number="l.cantidad" /></td>
              <td><input v-model="l.unidad" /></td>
              <td class="der"><input class="num interno" type="number" step="any" v-model.number="l.coste_unitario" @input="cambioCoste(l)" /></td>
              <td class="der"><input class="num" type="number" step="any" v-model.number="l.precio_unitario" @input="l.auto = false" /></td>
              <td class="der importe">{{ eur(totales?.lineas[i]?.importe) }}</td>
              <td><button class="peligro" @click="p.lineas.splice(i, 1)">×</button></td>
            </tr>
          </tbody>
        </table>
        <div class="fila" style="margin-top: 8px">
          <button @click="anadir('material')">+ Repuesto</button>
          <button @click="anadir('hora_oficial')">+ Hora oficial</button>
          <button @click="anadir('hora_pareja')">+ Oficial + ayudante</button>
          <button @click="anadir('salida')">+ Desplazamiento</button>
          <button @click="anadir('partida')">+ Otra partida</button>
          <span class="pequeño" v-if="recargo != null">Material: precio = coste + {{ recargo }} % (se puede cambiar a mano)</span>
        </div>

        <div class="fila" style="margin-top: 12px">
          <label class="campo">Descuento %<input class="num" type="number" step="any" v-model.number="p.descuento_pct" /></label>
          <label class="campo">IVA %<input class="num" type="number" step="any" v-model.number="p.iva_pct" /></label>
          <label class="campo">Validez (días)<input class="num" type="number" v-model.number="p.validez_dias" /></label>
        </div>
        <div v-if="totales" class="tarjetas" style="margin-top: 10px">
          <div class="tarjeta"><div class="etiqueta">Base imponible</div><div class="valor">{{ eur(totales.base) }}</div></div>
          <div class="tarjeta"><div class="etiqueta">IVA</div><div class="valor">{{ eur(totales.iva) }}</div></div>
          <div class="tarjeta destacada"><div class="etiqueta">Total</div><div class="valor">{{ eur(totales.total) }}</div></div>
          <div class="tarjeta interna">
            <div class="etiqueta">Margen interno</div>
            <div class="valor" :class="claseMargen">{{ eur(totales.margen) }} · {{ num(totales.margen_pct) }} %</div>
            <div v-if="totales.lineas_sin_coste" class="pequeño">{{ totales.lineas_sin_coste }} línea(s) sin coste</div>
          </div>
        </div>
      </div>

      <div class="panel" style="margin-top: 10px">
        <label class="campo bloque">Condiciones (vacío = las estándar: validez, plazo, pago, garantía y exclusiones)
          <textarea v-model="p.condiciones" rows="3" />
        </label>
        <label class="campo bloque">Notas internas (no salen en la carta)
          <textarea v-model="p.notas_internas" rows="2" />
        </label>
      </div>

      <div v-if="delCliente.length" class="panel" style="margin-top: 10px">
        <h2 style="margin-top: 0">Otros presupuestos de {{ p.cliente }}</h2>
        <table class="listado">
          <tbody>
            <tr v-for="r in delCliente" :key="r.numero" @click="abrir(r.numero)">
              <td style="width: 105px"><b>{{ r.numero }}</b></td><td style="width: 90px">{{ fecha(r.fecha) }}</td>
              <td>{{ r.titulo }}<div class="pequeño">{{ r.instalacion }}</div></td>
              <td style="width: 105px"><span :class="['estado', r.estado]">{{ ESTADOS[r.estado] }}</span></td>
              <td class="der" style="width: 110px">{{ eur(r.base) }}</td>
            </tr>
          </tbody>
        </table>
      </div>

      <div v-if="nuevoModelo" class="panel" style="margin-top: 10px">
        <h2 style="margin-top: 0">Guardar como modelo</h2>
        <p class="pequeño">Se guardan la explicación y las líneas, sin datos del cliente. La mano de obra y el desplazamiento
          tomarán siempre los precios del año.</p>
        <div class="fila">
          <label class="campo">Código<input v-model="nuevoModelo.codigo" placeholder="EXT-CORREAS" style="width: 150px" /></label>
          <label class="campo" style="flex: 1">Grupo de equipos<input v-model="nuevoModelo.grupo" list="lista-grupos" />
            <datalist id="lista-grupos"><option v-for="g in nombresGrupos" :key="g" :value="g" /></datalist></label>
          <label class="campo" style="flex: 2">Nombre<input v-model="nuevoModelo.nombre" /></label>
          <button @click="nuevoModelo = null">Cancelar</button>
          <button class="primario" @click="guardarModelo">Guardar modelo</button>
        </div>
      </div>

      <div class="fila" style="margin-top: 10px">
        <button @click="prepararModelo">Guardar como modelo…</button>
        <span style="flex: 1"></span>
        <button v-if="p.numero" class="peligro" @click="borrar">Borrar</button>
        <button @click="guardar">Guardar</button>
        <button class="primario" @click="pdf">Guardar y carta PDF</button>
      </div>
    </div>
</template>

<style scoped>
.cabecera { margin-bottom: 10px; }
.modelos { margin-bottom: 10px; }
.ambito.propio { color: var(--verde); background: #e8f4ea; }
.ambito { display: inline-block; font-size: 10px; font-weight: 600; color: var(--azul); background: var(--azul-claro); border-radius: 4px; padding: 0 5px; margin-top: 2px; }
.instalacion { margin: 12px 0 4px; font-weight: 600; color: var(--azul); font-size: 13px; border-bottom: 1px solid var(--borde); padding-bottom: 3px; }
.listado tr { cursor: pointer; }
.listado tr:hover td { background: var(--fondo); }
.listado td { vertical-align: middle; }
.est-enviado { background: #e8eef6; }
.est-aceptado { background: #e8f4ea; }
.est-rechazado { background: #fdecea; }
.est-borrador { background: var(--fondo); border: 1px solid var(--borde); }
.dos { display: grid; grid-template-columns: 280px 1fr; gap: 16px; align-items: start; }
.dos > div { min-width: 0; }
.lista ul { list-style: none; padding: 0; margin: 10px 0 0; }
.lista li { padding: 8px; border-radius: 6px; cursor: pointer; display: flex; flex-direction: column; gap: 2px; }
.lista li:hover { background: var(--fondo); }
.lista li.sel { background: var(--azul-claro); }
.entre { justify-content: space-between; }
.estado { font-size: 11px; padding: 1px 7px; border-radius: 10px; background: var(--fondo); color: var(--gris); }
.estado.enviado { background: #e8eef6; color: var(--azul); }
.estado.aceptado { background: #e8f4ea; color: var(--verde); }
.estado.rechazado { background: #fdecea; color: var(--rojo); }
.bloque { margin-top: 8px; }
textarea { font: inherit; border: 1px solid var(--borde); border-radius: 5px; padding: 6px 8px; width: 100%; resize: vertical; color: var(--texto); }
.lineas { table-layout: fixed; }
.lineas select, .lineas input:not(.num) { width: 100%; }
.lineas input.num { width: 100%; }
.lineas input.interno { background: #fafbfc; color: var(--gris); }
.importe { padding-top: 10px; }
.tarjeta.interna { background: #fafbfc; border: 1px dashed var(--borde); }
.tarjeta.interna .valor { font-size: 16px; }
</style>
