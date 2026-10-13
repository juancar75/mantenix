<!--
  Contratos: lista, edición del inventario y valoración.

  Flujo: seleccionar o crear contrato → editar datos e inventario → Guardar → Valorar → PDF.
  Valorar siempre trabaja sobre lo guardado en la base de datos, para que la valoración que se
  guarda en el historial corresponda exactamente al inventario registrado.
-->
<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import { api, eur, num, type Contrato, type EquipoVista, type Valoracion } from "../api";

const emit = defineEmits<{ cambio: [] }>();

const lista = ref<{ nombre: string; anio: number; precio: number | null }[]>([]);
const catalogo = ref<EquipoVista[]>([]);
const contrato = ref<Contrato | null>(null);
const nombreGuardado = ref<string | null>(null);
const valoracion = ref<Valoracion | null>(null);
const error = ref("");
const mensaje = ref("");

const descripcionDe = computed(() => Object.fromEntries(catalogo.value.map((e) => [e.codigo, e.descripcion])));

async function cargarLista() { lista.value = await api.listarContratos(); }

onMounted(async () => {
  catalogo.value = await api.catalogo();
  await cargarLista();
});

/** Ejecuta una acción mostrando el error, si lo hay, en la cabecera. */
async function intentar(f: () => Promise<void>) {
  error.value = ""; mensaje.value = "";
  try { await f(); } catch (e) { error.value = String(e); }
}

const abrir = (nombre: string) => intentar(async () => {
  contrato.value = await api.cargarContrato(nombre);
  nombreGuardado.value = nombre;
  valoracion.value = null;
});

function nuevo() {
  contrato.value = {
    nombre: "", anio: new Date().getFullYear(), uso: "otros", visitas_contratadas: null,
    desplazamiento: { km_ida: 0, minutos_ida: 0 }, instalaciones: [], margen_pct: null, precio_actual_anual: null,
  };
  nombreGuardado.value = null;
  valoracion.value = null;
}

function anadirInstalacion(familia: string) {
  contrato.value?.instalaciones.push({ familia, titulo: familia === "RITE" ? "Climatización" : "Protección contra incendios", potencia_kw: familia === "RITE" ? 0 : null, temporada_meses: 12, equipos: [] });
}

const guardar = () => intentar(async () => {
  if (!contrato.value) return;
  await api.guardarContrato(contrato.value);
  nombreGuardado.value = contrato.value.nombre;
  mensaje.value = "Contrato guardado.";
  await cargarLista(); emit("cambio");
});

const valorar = () => intentar(async () => {
  if (!contrato.value) return;
  await api.guardarContrato(contrato.value);
  nombreGuardado.value = contrato.value.nombre;
  valoracion.value = await api.valorar(contrato.value.nombre);
  await cargarLista();
});

const pdf = () => intentar(async () => {
  if (!nombreGuardado.value) return;
  const ruta = await save({ defaultPath: `hoja-costes-${nombreGuardado.value}.pdf`, filters: [{ name: "PDF", extensions: ["pdf"] }] });
  if (!ruta) return;
  await api.pdfHojaCostes(nombreGuardado.value, ruta);
  mensaje.value = `PDF guardado en ${ruta}`;
});

const borrar = () => intentar(async () => {
  if (!nombreGuardado.value || !confirm(`¿Borrar el contrato «${nombreGuardado.value}» y sus valoraciones?`)) return;
  await api.borrarContrato(nombreGuardado.value);
  contrato.value = null; nombreGuardado.value = null; valoracion.value = null;
  await cargarLista(); emit("cambio");
});

const margenClase = (v: Valoracion) =>
  v.totales.margen_precio_actual_pct == null ? "" : v.totales.margen_precio_actual_pct < v.totales.margen_pct ? "malo" : "bueno";
</script>

<template>
  <h1>Contratos</h1>
  <div v-if="error" class="error">{{ error }}</div>
  <div v-if="mensaje" class="aviso">{{ mensaje }}</div>

  <div class="dos">
    <div class="panel lista">
      <button class="primario" @click="nuevo">+ Nuevo contrato</button>
      <ul>
        <li v-for="c in lista" :key="c.nombre" :class="{ sel: c.nombre === nombreGuardado }" @click="abrir(c.nombre)">
          <div>{{ c.nombre }}</div>
          <div class="pequeño">{{ c.anio }} · {{ c.precio == null ? "sin valorar" : eur(c.precio) }}</div>
        </li>
      </ul>
    </div>

    <div v-if="contrato">
      <div class="panel">
        <div class="fila">
          <label class="campo" style="flex: 1">Nombre<input v-model="contrato.nombre" /></label>
          <label class="campo">Año<input class="num" type="number" v-model.number="contrato.anio" /></label>
          <label class="campo">Uso
            <select v-model="contrato.uso"><option value="otros">Otros usos</option><option value="viviendas">Viviendas</option></select>
          </label>
          <label class="campo">Visitas/año<input class="num" type="number" v-model.number="contrato.visitas_contratadas" placeholder="mínimo" /></label>
        </div>
        <div class="fila" style="margin-top: 8px">
          <label class="campo">Km ida<input class="num" type="number" v-model.number="contrato.desplazamiento.km_ida" /></label>
          <label class="campo">Minutos ida<input class="num" type="number" v-model.number="contrato.desplazamiento.minutos_ida" /></label>
          <label class="campo">Margen % (vacío = el del año)<input class="num" type="number" v-model.number="contrato.margen_pct" /></label>
          <label class="campo">Precio actual €/año<input class="num" type="number" v-model.number="contrato.precio_actual_anual" /></label>
        </div>
      </div>

      <div v-for="(ins, i) in contrato.instalaciones" :key="i" class="panel" style="margin-top: 10px">
        <div class="fila">
          <b>{{ ins.familia }}</b>
          <input v-model="ins.titulo" style="flex: 1" />
          <label v-if="ins.familia === 'RITE'" class="campo">Potencia kW<input class="num" type="number" v-model.number="ins.potencia_kw" /></label>
          <label v-if="ins.familia === 'RITE'" class="campo">Meses de uso<input class="num" type="number" v-model.number="ins.temporada_meses" /></label>
          <button class="peligro" @click="contrato.instalaciones.splice(i, 1)">Quitar instalación</button>
        </div>
        <table class="inventario" style="margin-top: 8px">
          <colgroup><col style="width: 38%" /><col /><col style="width: 100px" /><col style="width: 150px" /><col style="width: 44px" /></colgroup>
          <thead><tr><th>Equipo</th><th>Descripción</th><th class="der">Cantidad</th><th>Modo</th><th></th></tr></thead>
          <tbody>
            <tr v-for="(l, j) in ins.equipos" :key="j">
              <td>
                <select v-model="l.codigo">
                  <option v-for="e in catalogo" :key="e.codigo" :value="e.codigo">{{ e.codigo }} — {{ e.descripcion }}</option>
                </select>
              </td>
              <td><input v-model="l.descripcion" :placeholder="descripcionDe[l.codigo]" style="width: 100%" /></td>
              <td class="der"><input class="num" type="number" v-model.number="l.cantidad" /></td>
              <td>
                <select v-model="l.modo">
                  <option :value="null">Por defecto</option><option value="propio">Medios propios</option><option value="subcontrata">Subcontrata</option>
                </select>
              </td>
              <td><button class="peligro" @click="ins.equipos.splice(j, 1)">×</button></td>
            </tr>
          </tbody>
        </table>
        <button style="margin-top: 6px" @click="ins.equipos.push({ codigo: catalogo[0]?.codigo ?? '', cantidad: 1, descripcion: '', modo: null })">+ Equipo</button>
      </div>

      <div class="fila" style="margin-top: 10px">
        <button @click="anadirInstalacion('RITE')">+ Instalación RITE</button>
        <button @click="anadirInstalacion('PCI')">+ Instalación PCI</button>
        <span style="flex: 1"></span>
        <button v-if="nombreGuardado" class="peligro" @click="borrar">Borrar</button>
        <button @click="guardar">Guardar</button>
        <button class="primario" @click="valorar">Guardar y valorar</button>
        <button v-if="valoracion" @click="pdf">Hoja de costes PDF</button>
      </div>

      <template v-if="valoracion">
        <h2>Valoración {{ valoracion.anio }}</h2>
        <div class="tarjetas">
          <div class="tarjeta destacada"><div class="etiqueta">Precio anual</div><div class="valor">{{ eur(valoracion.totales.precio_anual) }}</div></div>
          <div class="tarjeta"><div class="etiqueta">Por visita</div><div class="valor">{{ eur(valoracion.totales.precio_visita) }}</div></div>
          <div class="tarjeta"><div class="etiqueta">Coste total</div><div class="valor">{{ eur(valoracion.totales.coste_total) }}</div></div>
          <div class="tarjeta"><div class="etiqueta">Visitas (mín.)</div><div class="valor">{{ valoracion.visitas }} ({{ valoracion.visitas_minimas }})</div></div>
        </div>
        <p v-if="valoracion.totales.precio_actual_anual != null">
          Precio actual {{ eur(valoracion.totales.precio_actual_anual) }} → margen real
          <span :class="margenClase(valoracion)">{{ num(valoracion.totales.margen_precio_actual_pct ?? 0) }} %</span>
          (objetivo {{ valoracion.totales.margen_pct }} %)
        </p>
        <div v-for="a in valoracion.avisos" :key="a" class="aviso">{{ a }}</div>
        <ul class="pequeño"><li v-for="r in valoracion.regimenes" :key="r.instalacion">{{ r.instalacion }}: {{ r.descripcion }}</li></ul>

        <table>
          <thead><tr><th>Concepto</th><th class="der">Horas</th><th class="der">Importe</th></tr></thead>
          <tbody>
            <tr><td>Mano de obra en equipos</td><td class="der">{{ num(valoracion.totales.horas_equipos) }}</td><td class="der">{{ eur(valoracion.totales.mano_obra) }}</td></tr>
            <tr><td>Desplazamiento y tiempo fijo</td><td class="der">{{ num(valoracion.totales.horas_desplazamiento) }}</td><td class="der">{{ eur(valoracion.totales.desplazamiento) }}</td></tr>
            <tr><td>Consumibles</td><td></td><td class="der">{{ eur(valoracion.totales.consumibles) }}</td></tr>
            <tr><td>Subcontrata ({{ eur(valoracion.subcontrata_visita_facturada) }}/visita × {{ valoracion.visitas_subcontrata }})</td><td></td><td class="der">{{ eur(valoracion.totales.subcontrata) }}</td></tr>
            <tr><td><b>Coste directo</b></td><td></td><td class="der"><b>{{ eur(valoracion.totales.directo) }}</b></td></tr>
            <tr><td>Indirectos + contingencia</td><td></td><td class="der">{{ eur(valoracion.totales.indirectos + valoracion.totales.contingencia) }}</td></tr>
            <tr><td><b>Precio de venta</b></td><td></td><td class="der"><b>{{ eur(valoracion.totales.precio_anual) }}</b></td></tr>
          </tbody>
        </table>

        <h2>Por equipo</h2>
        <table>
          <thead><tr><th>Equipo</th><th class="der">Cant.</th><th>Modo</th><th class="der">Horas/año</th><th class="der">Mano de obra</th><th class="der">Subcontr./visita</th></tr></thead>
          <tbody>
            <tr v-for="(l, k) in valoracion.lineas" :key="k">
              <td>{{ l.descripcion }} <span class="pequeño">{{ l.codigo }}</span></td>
              <td class="der">{{ l.cantidad }}</td><td>{{ l.modo }}</td>
              <td class="der">{{ l.modo === "propio" ? num(l.horas_anio) : "—" }}</td>
              <td class="der">{{ l.modo === "propio" ? eur(l.mano_obra) : "—" }}</td>
              <td class="der">{{ l.modo === "subcontrata" ? eur(l.subcontrata_visita) : "—" }}</td>
            </tr>
          </tbody>
        </table>
      </template>
    </div>
    <div v-else class="panel pequeño">Seleccione un contrato o cree uno nuevo.</div>
  </div>
</template>

<style scoped>
.dos { display: grid; grid-template-columns: 280px 1fr; gap: 16px; align-items: start; }
.lista ul { list-style: none; padding: 0; margin: 10px 0 0; }
.lista li { padding: 8px; border-radius: 6px; cursor: pointer; }
.lista li:hover { background: var(--fondo); }
.lista li.sel { background: var(--azul-claro); }
.inventario { table-layout: fixed; }
.inventario select, .inventario input:not(.num) { width: 100%; }
.dos > div { min-width: 0; }
</style>
