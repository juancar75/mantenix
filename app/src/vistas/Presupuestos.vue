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
  type LineaPresupuesto, type PrecioDefecto, type Presupuesto, type ResumenPresupuesto, type TotalesPresupuesto,
} from "../api";

const emit = defineEmits<{ cambio: [] }>();

const ESTADOS: Record<string, string> = { borrador: "Borrador", enviado: "Enviado", aceptado: "Aceptado", rechazado: "Rechazado" };
const TIPOS_LINEA: Record<string, string> = {
  material: "Repuesto / material", hora_oficial: "Hora oficial", hora_pareja: "Hora oficial + ayudante",
  salida: "Desplazamiento", partida: "Otra partida",
};

const lista = ref<ResumenPresupuesto[]>([]);
const filtro = ref("");
const contratos = ref<string[]>([]);
const p = ref<Presupuesto | null>(null);
const totales = ref<TotalesPresupuesto | null>(null);
const precios = ref<PrecioDefecto[]>([]);
const recargo = ref<number | null>(null);
const error = ref("");
const mensaje = ref("");

const hoy = () => new Date().toISOString().slice(0, 10);
const anio = computed(() => Number(p.value?.fecha.slice(0, 4)) || new Date().getFullYear());

const listaFiltrada = computed(() => {
  const f = filtro.value.trim().toLowerCase();
  return f ? lista.value.filter((r) => `${r.numero} ${r.cliente} ${r.titulo}`.toLowerCase().includes(f)) : lista.value;
});

async function cargarLista() { lista.value = await api.listarPresupuestos(); }
async function cargarPrecios() {
  const r = await api.preciosPresupuesto(anio.value);
  precios.value = r.filas; recargo.value = r.recargo_material_pct;
}

onMounted(async () => {
  await cargarLista();
  contratos.value = (await api.listarContratos()).map((c) => c.nombre);
});

async function intentar(f: () => Promise<void>) {
  error.value = ""; mensaje.value = "";
  try { await f(); } catch (e) { error.value = String(e); }
}

const abrir = (numero: string) => intentar(async () => {
  p.value = await api.cargarPresupuesto(numero);
  p.value.lineas.forEach((l) => (l.auto = false));
  await cargarPrecios();
});

const nuevo = (tipo: string) => intentar(async () => {
  p.value = {
    numero: "", fecha: hoy(), tipo, cliente: "", cif: "", direccion: "", contacto: "", contrato: "",
    titulo: "", equipo: "", deteccion: "", averia: "", consecuencias: "", propuesta: "",
    lineas: [], descuento_pct: 0, iva_pct: 21, validez_dias: 30, condiciones: "", estado: "borrador", notas_internas: "",
  };
  await cargarPrecios();
  anadir("material");
  anadir(tipo === "correctivo" ? "hora_oficial" : "hora_pareja");
  anadir("salida");
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
  p.value = null; totales.value = null;
  await cargarLista(); emit("cambio");
});

const claseMargen = computed(() => !totales.value ? "" : totales.value.margen_pct < 15 ? "malo" : "bueno");
</script>

<template>
  <h1>Presupuestos</h1>
  <div v-if="error" class="error">{{ error }}</div>
  <div v-if="mensaje" class="aviso">{{ mensaje }}</div>

  <div class="dos">
    <div class="panel lista">
      <div class="fila">
        <button class="primario" @click="nuevo('correctivo')">+ Reparación</button>
        <button @click="nuevo('obra')">+ Obra / mejora</button>
      </div>
      <input v-model="filtro" placeholder="Buscar…" style="width: 100%; margin-top: 8px" />
      <ul>
        <li v-for="r in listaFiltrada" :key="r.numero" :class="{ sel: r.numero === p?.numero }" @click="abrir(r.numero)">
          <div class="fila entre"><b>{{ r.numero }}</b><span :class="['estado', r.estado]">{{ ESTADOS[r.estado] }}</span></div>
          <div>{{ r.titulo }}</div>
          <div class="pequeño">{{ r.cliente }} · {{ eur(r.total) }}</div>
        </li>
        <li v-if="!lista.length" class="pequeño">Todavía no hay presupuestos.</li>
      </ul>
    </div>

    <div v-if="p">
      <div class="panel">
        <div class="fila">
          <label class="campo">Número<input :value="p.numero || 'al guardar'" disabled style="width: 120px" /></label>
          <label class="campo">Fecha<input type="date" v-model="p.fecha" /></label>
          <label class="campo">Tipo
            <select v-model="p.tipo"><option value="correctivo">Reparación</option><option value="obra">Obra / mejora</option></select>
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
          <label class="campo" style="flex: 2">Cliente<input v-model="p.cliente" /></label>
          <label class="campo">CIF/NIF<input v-model="p.cif" style="width: 120px" /></label>
          <label class="campo" style="flex: 1">A la atención de<input v-model="p.contacto" /></label>
        </div>
        <div class="fila" style="margin-top: 8px">
          <label class="campo" style="flex: 1">Dirección de la instalación<input v-model="p.direccion" /></label>
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

      <div class="fila" style="margin-top: 10px">
        <span style="flex: 1"></span>
        <button v-if="p.numero" class="peligro" @click="borrar">Borrar</button>
        <button @click="guardar">Guardar</button>
        <button class="primario" @click="pdf">Guardar y carta PDF</button>
      </div>
    </div>
    <div v-else class="panel pequeño">
      Seleccione un presupuesto o cree uno nuevo. La carta explica al cliente qué se ha roto o desgastado,
      sus consecuencias y el repuesto que se propone, con el importe y un espacio para su aceptación.
    </div>
  </div>
</template>

<style scoped>
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
