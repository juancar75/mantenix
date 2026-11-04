<!--
  Asistente «Registrar visita»: guía al técnico o al responsable de mantenimiento paso a paso y
  deja la visita en el registro (RITE art. 27, RIPCI art. 21.2).

  Pasos: contrato y visita → datos de la visita → operaciones (bien / defecto / no aplica) →
  medidas y contadores → defectos y material → guardar (parte en PDF y, si hay defectos,
  presupuesto del correctivo ya rellenado).
-->
<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import { api, type PlanVista, type Registro, type VisitaPlan } from "../api";
import { ir, nav } from "../navegacion";

const PASOS = ["Contrato y visita", "Datos", "Operaciones", "Medidas y contadores", "Defectos y material", "Guardar"];
const CONTADORES = [
  { clave: "combustible", nombre: "Combustible (m³, kg o L)" },
  { clave: "electricidad", nombre: "Electricidad (kWh)" },
  { clave: "agua_llenado", nombre: "Agua de llenado (m³)" },
  { clave: "acs", nombre: "ACS (m³)" },
  { clave: "energia_termica", nombre: "Energía térmica (kWh)" },
];

const paso = ref(0);
const error = ref("");
const mensaje = ref("");
const contratos = ref<string[]>([]);
const contrato = ref("");
const pv = ref<PlanVista | null>(null);
const numero = ref(0); // 0 = aviso o reparación
const tipo = ref("preventiva");
const guardadoId = ref<string | null>(null);

const hoy = new Date().toISOString().slice(0, 10);
const leer = (k: string) => { try { return localStorage.getItem(k) ?? ""; } catch { return ""; } };
const d = reactive({ fecha: hoy, tecnico: leer("mantenix.tecnico"), entrada: "", salida: "" });
// clave «equipo|operación» → {resultado, nota}
const res = reactive<Record<string, { resultado: string; nota: string }>>({});
const medidas = reactive<Record<string, Record<string, string>>>({});
const contadores = reactive<Record<string, number | null>>({});
const cierre = reactive({ defectos: "", material: "", observaciones: "" });

onMounted(async () => {
  contratos.value = (await api.listarContratos()).map((c) => c.nombre);
  if (nav.visita) {
    contrato.value = nav.visita.contrato;
    await cargarPlan(nav.visita.numero);
    nav.visita = null;
  }
});

async function cargarPlan(elegir = -1) {
  error.value = ""; pv.value = null;
  if (!contrato.value) return;
  try {
    pv.value = await api.planContrato(contrato.value);
    const sig = pv.value.estado.find((e) => e.estado === "atrasada") ?? pv.value.estado.find((e) => e.estado === "pendiente");
    numero.value = elegir > 0 ? elegir : sig?.numero ?? 1;
    tipo.value = "preventiva";
  } catch (e) { error.value = String(e); }
}

const visita = computed<VisitaPlan | null>(() => pv.value?.plan.visitas.find((v) => v.numero === numero.value) ?? null);
const estadoDe = (n: number) => pv.value?.estado.find((e) => e.numero === n);
const clave = (eq: string, op: string) => `${eq}|${op}`;
const ops = computed(() => (visita.value?.equipos ?? []).flatMap((q) => q.operaciones.map((o) => ({ q, o, k: clave(q.descripcion, o.operacion_id) }))));
const hayMedidas = computed(() => tipo.value === "preventiva" && ((visita.value?.medidas.length ?? 0) > 0 || visita.value?.contadores));

function preparar() {
  for (const k of Object.keys(res)) delete res[k];
  for (const x of ops.value) res[x.k] = { resultado: "", nota: "" };
  for (const k of Object.keys(medidas)) delete medidas[k];
  for (const m of visita.value?.medidas ?? []) for (const g of m.generadores) medidas[g] = medidas[g] ?? {};
}
function todoBien(eq?: string) {
  for (const x of ops.value) if ((!eq || x.q.descripcion === eq) && !res[x.k].resultado) res[x.k].resultado = "B";
}
const defectos = computed(() => ops.value.filter((x) => res[x.k]?.resultado === "D"));

const falta = computed<string>(() => {
  if (paso.value === 0) {
    if (!contrato.value) return "Elija el contrato.";
    if (tipo.value === "preventiva" && !visita.value) return "Elija la visita.";
  }
  if (paso.value === 1 && !d.fecha) return "Indique la fecha.";
  if (paso.value === 1 && !d.tecnico.trim()) return "Indique quién hace la visita.";
  if (paso.value === 2) {
    const sin = ops.value.filter((x) => !res[x.k]?.resultado).length;
    if (sin) return `Quedan ${sin} operaciones sin marcar (use «Todo bien» en cada equipo si procede).`;
    const sinNota = defectos.value.filter((x) => !res[x.k].nota.trim()).length;
    if (sinNota) return "Describa cada defecto en su nota.";
  }
  if (paso.value === 4 && tipo.value !== "preventiva" && !cierre.defectos.trim() && !cierre.observaciones.trim())
    return "Describa la avería o el trabajo hecho.";
  return "";
});

function siguiente() {
  error.value = "";
  if (falta.value) { error.value = falta.value; return; }
  if (paso.value === 0) { if (tipo.value !== "preventiva") numero.value = 0; preparar(); }
  if (paso.value === 1) { try { localStorage.setItem("mantenix.tecnico", d.tecnico); } catch { /* sin almacenamiento */ } }
  if (paso.value === 2 && !cierre.defectos)
    cierre.defectos = defectos.value.map((x) => `${x.q.descripcion}: ${res[x.k].nota}`).join("\n");
  paso.value++;
  // Saltar pasos que no aplican.
  if (paso.value === 2 && tipo.value !== "preventiva") paso.value = 4;
  if (paso.value === 3 && !hayMedidas.value) paso.value = 4;
  if (paso.value === 5) guardar();
}
function anterior() {
  error.value = "";
  paso.value--;
  if (paso.value === 3 && !hayMedidas.value) paso.value = 2;
  if (paso.value === 2 && tipo.value !== "preventiva") paso.value = 1;
}

async function guardar() {
  error.value = "";
  const r: Registro = {
    id: guardadoId.value ?? "", contrato: contrato.value, anio: pv.value?.plan.anio ?? Number(d.fecha.slice(0, 4)),
    numero: tipo.value === "preventiva" ? numero.value : 0, tipo: tipo.value, fecha: d.fecha,
    tecnico: d.tecnico.trim(), entrada: d.entrada, salida: d.salida,
    resultados: ops.value.map((x) => ({ operacion_id: x.o.operacion_id, equipo: x.q.descripcion, texto: x.o.texto, resultado: res[x.k].resultado, nota: res[x.k].nota.trim() })),
    medidas: Object.entries(medidas).map(([generador, valores]) => ({ generador, valores })),
    contadores: Object.fromEntries(Object.entries(contadores).filter(([, v]) => v != null && !Number.isNaN(v)).map(([k, v]) => [k, Number(v)])),
    defectos: cierre.defectos.trim(), material: cierre.material.trim(), observaciones: cierre.observaciones.trim(),
  };
  try { guardadoId.value = await api.guardarRegistro(r); }
  catch (e) { error.value = String(e); paso.value = 4; }
}

async function parte() {
  if (!guardadoId.value) return;
  error.value = ""; mensaje.value = "";
  try {
    const ruta = await save({ defaultPath: `parte-${contrato.value}-${d.fecha}.pdf`, filters: [{ name: "PDF", extensions: ["pdf"] }] });
    if (!ruta) return;
    await api.pdfParte(contrato.value, guardadoId.value, ruta);
    mensaje.value = `Parte guardado en ${ruta}`;
  } catch (e) { error.value = String(e); }
}

function correctivo(texto: string, equipo: string) {
  nav.correctivo = { contrato: contrato.value, cliente: "", direccion: "", ambito: "", equipo, averia: texto };
  ir("presupuestos");
}

function otra() {
  guardadoId.value = null; paso.value = 0; cierre.defectos = ""; cierre.material = ""; cierre.observaciones = "";
  for (const k of Object.keys(contadores)) delete contadores[k];
  cargarPlan();
}
</script>

<template>
  <h1>Registrar visita · asistente</h1>
  <ol class="pasos">
    <li v-for="(p, i) in PASOS" :key="p" :class="{ actual: i === paso, hecho: i < paso }"><span class="n">{{ i + 1 }}</span>{{ p }}</li>
  </ol>
  <div v-if="error" class="error">{{ error }}</div>
  <div v-if="mensaje" class="ok">{{ mensaje }}</div>

  <!-- 1. Contrato y visita -->
  <div v-if="paso === 0" class="panel paso">
    <h2>¿Dónde ha estado y qué ha hecho?</h2>
    <label class="campo">Contrato
      <select v-model="contrato" @change="cargarPlan()"><option value="" disabled>Elija…</option><option v-for="c in contratos" :key="c">{{ c }}</option></select></label>
    <div class="opciones">
      <label :class="{ sel: tipo === 'preventiva' }"><input type="radio" v-model="tipo" value="preventiva" />Visita de mantenimiento del plan</label>
      <label :class="{ sel: tipo === 'aviso' }"><input type="radio" v-model="tipo" value="aviso" />Aviso de avería</label>
      <label :class="{ sel: tipo === 'reparacion' }"><input type="radio" v-model="tipo" value="reparacion" />Reparación o correctivo</label>
    </div>
    <template v-if="pv && tipo === 'preventiva'">
      <p class="ayuda">Elija la visita. En rojo, las atrasadas; en verde, las ya hechas (si elige una hecha, se añade otro registro).</p>
      <div class="visitas">
        <button v-for="e in pv.estado" :key="e.numero" :class="['vis', e.estado, { sel: numero === e.numero }]" @click="numero = e.numero">
          <b>{{ e.numero }}</b> {{ e.mes_nombre }}<br /><span class="pequeño">{{ e.fecha ?? e.estado }}{{ e.defectos ? ` · ${e.defectos} def.` : "" }}</span>
        </button>
      </div>
      <p v-if="visita" class="ayuda">Visita {{ visita.numero }}: {{ visita.equipos.length }} equipos, {{ visita.horas.toString().replace(".", ",") }} h previstas{{ visita.medidas.length ? ", con medidas de generadores" : "" }}{{ visita.subcontrata.length ? ", y revisión de PCI de la subcontrata" : "" }}.</p>
    </template>
  </div>

  <!-- 2. Datos -->
  <div v-if="paso === 1" class="panel paso">
    <h2>Datos de la visita</h2>
    <div class="fila">
      <label class="campo">Fecha<input type="date" v-model="d.fecha" /></label>
      <label class="campo" style="flex: 1">Técnico (nombre y n.º de carné)<input v-model="d.tecnico" /></label>
      <label class="campo">Entrada<input type="time" v-model="d.entrada" /></label>
      <label class="campo">Salida<input type="time" v-model="d.salida" /></label>
    </div>
  </div>

  <!-- 3. Operaciones -->
  <div v-if="paso === 2 && visita" class="paso">
    <div class="panel fila"><span class="ayuda" style="flex: 1">Marque cada operación: <b>Bien</b>, <b>Defecto</b> (escriba qué pasa) o <b>No aplica</b>.</span>
      <button @click="todoBien()">Todo lo que falta, bien</button></div>
    <div v-for="q in visita.equipos" :key="q.descripcion" class="panel equipo">
      <div class="fila"><h3 style="flex: 1">{{ q.descripcion }} <span class="pequeño">× {{ q.cantidad }}</span></h3>
        <button @click="todoBien(q.descripcion)">Todo bien</button></div>
      <div v-for="o in q.operaciones" :key="o.operacion_id" class="op" :class="res[clave(q.descripcion, o.operacion_id)]?.resultado">
        <div class="texto">{{ o.texto }} <span v-if="o.nota" class="pequeño">— {{ o.nota }}</span></div>
        <div class="botones">
          <button v-for="b in [['B', 'Bien'], ['D', 'Defecto'], ['NA', 'No aplica']]" :key="b[0]"
            :class="['res', b[0], { activo: res[clave(q.descripcion, o.operacion_id)].resultado === b[0] }]"
            @click="res[clave(q.descripcion, o.operacion_id)].resultado = b[0]">{{ b[1] }}</button>
        </div>
        <input v-if="res[clave(q.descripcion, o.operacion_id)].resultado === 'D'" v-model="res[clave(q.descripcion, o.operacion_id)].nota"
          class="nota" placeholder="¿Qué pasa? (p. ej. precarga del vaso a 0,8 bar)" />
      </div>
    </div>
  </div>

  <!-- 4. Medidas y contadores -->
  <div v-if="paso === 3 && visita" class="panel paso">
    <template v-for="m in visita.medidas" :key="m.tipo">
      <h2>Medidas de generadores de {{ m.tipo }} (IT 3.4 del RITE)</h2>
      <table>
        <thead><tr><th>Generador</th><th v-for="g in m.magnitudes" :key="g">{{ g }}</th></tr></thead>
        <tbody><tr v-for="gen in m.generadores" :key="gen"><td>{{ gen }}</td>
          <td v-for="g in m.magnitudes" :key="g"><input v-model="medidas[gen][g]" class="medida" /></td></tr></tbody>
      </table>
    </template>
    <template v-if="visita.contadores">
      <h2>Lectura de contadores (IT 3.4.4)</h2>
      <p class="ayuda">Anote la lectura tal como sale en el contador (acumulada). El programa calcula el consumo del año para el certificado.</p>
      <div class="fila"><label v-for="c in CONTADORES" :key="c.clave" class="campo">{{ c.nombre }}
        <input class="num" type="number" v-model.number="contadores[c.clave]" /></label></div>
    </template>
  </div>

  <!-- 5. Defectos y material -->
  <div v-if="paso === 4" class="panel paso">
    <h2>{{ tipo === 'preventiva' ? "Defectos, reparaciones y material" : "Avería y trabajo realizado" }}</h2>
    <label class="campo">{{ tipo === 'preventiva' ? "Defectos encontrados y acción propuesta" : "Qué fallaba y qué se ha hecho" }}
      <textarea v-model="cierre.defectos" rows="4" /></label>
    <label class="campo">Material sustituido o empleado<textarea v-model="cierre.material" rows="2" /></label>
    <label class="campo">Observaciones<textarea v-model="cierre.observaciones" rows="2" /></label>
  </div>

  <!-- 6. Guardar -->
  <div v-if="paso === 5" class="panel paso">
    <template v-if="guardadoId">
      <h2>Visita registrada</h2>
      <p>Queda en el registro de mantenimiento de <b>{{ contrato }}</b> del {{ d.fecha }}. Imprima el parte para que lo firme el cliente.</p>
      <div class="fila">
        <button class="primario" @click="parte">Parte de la visita (PDF)</button>
        <button @click="ir('inicio')">Volver a «Qué toca»</button>
        <button @click="otra">Registrar otra</button>
      </div>
      <template v-if="defectos.length">
        <h3>Defectos: ¿preparo el presupuesto del correctivo?</h3>
        <div v-for="x in defectos" :key="x.k" class="fila linea">
          <span style="flex: 1"><b>{{ x.q.descripcion }}</b>: {{ res[x.k].nota }}</span>
          <button @click="correctivo(res[x.k].nota, x.q.descripcion)">Presupuesto del correctivo</button>
        </div>
      </template>
    </template>
    <p v-else>Guardando…</p>
  </div>

  <div class="navegar" v-if="paso < 5">
    <button v-if="paso > 0" @click="anterior">← Anterior</button>
    <span style="flex: 1"></span>
    <button class="primario" @click="siguiente">{{ paso === 4 ? "Guardar en el registro" : "Siguiente →" }}</button>
  </div>
</template>

<style scoped>
.pasos { list-style: none; display: flex; flex-wrap: wrap; gap: 6px; padding: 0; margin: 0 0 14px; }
.pasos li { display: flex; align-items: center; gap: 6px; font-size: 12px; color: var(--gris); padding: 4px 10px 4px 4px; border-radius: 14px; background: white; border: 1px solid var(--borde); }
.pasos .n { display: inline-flex; width: 20px; height: 20px; border-radius: 50%; align-items: center; justify-content: center; background: var(--fondo); font-weight: 700; font-size: 11px; }
.pasos li.actual { color: var(--azul); border-color: var(--azul); font-weight: 600; }
.pasos li.actual .n { background: var(--azul); color: white; }
.pasos li.hecho .n { background: #2e7d32; color: white; }
.paso { margin-bottom: 12px; }
.visitas { display: grid; grid-template-columns: repeat(6, 1fr); gap: 6px; margin: 8px 0; }
.vis { text-align: left; padding: 6px 8px; border-radius: 6px; border: 1px solid var(--borde); background: white; text-transform: capitalize; }
.vis.hecha { background: #e3f4e6; }
.vis.atrasada { background: #fbe4e1; }
.vis.sel { outline: 2px solid var(--azul); }
.equipo { margin-bottom: 10px; }
.equipo h3 { margin: 0 0 6px; }
.op { display: grid; grid-template-columns: 1fr auto; gap: 4px 10px; padding: 6px 0; border-top: 1px solid var(--borde); }
.op.D { background: #fff4f2; }
.op .nota { grid-column: 1 / -1; }
.botones { display: flex; gap: 4px; }
.res { font-size: 12px; padding: 4px 9px; }
.res.activo.B { background: #2e7d32; color: white; border-color: #2e7d32; }
.res.activo.D { background: #c0392b; color: white; border-color: #c0392b; }
.res.activo.NA { background: #777; color: white; border-color: #777; }
.medida { width: 100%; min-width: 70px; }
textarea { width: 100%; font: inherit; }
.navegar { display: flex; gap: 8px; margin-top: 12px; }
.linea { padding: 6px 0; border-bottom: 1px solid var(--borde); align-items: center; }
.opciones { display: flex; gap: 8px; flex-wrap: wrap; margin: 10px 0; }
.opciones label { border: 1px solid var(--borde); border-radius: 6px; padding: 6px 10px; cursor: pointer; background: white; }
.opciones label.sel { border-color: var(--azul); background: var(--azul-claro); }
</style>
