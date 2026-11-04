<!--
  Inicio: qué toca hacer y calendario de mantenimiento de todos los contratos.

  Arriba, lo urgente (visitas atrasadas, las de este mes, inspecciones y certificados próximos),
  cada una con su botón al asistente que corresponde. Abajo, el calendario mensual. El calendario
  se exporta a .ics para importarlo en Outlook (se ve también en Teams) o en Google Calendar.
-->
<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import { api, type Evento } from "../api";
import { ir, nav } from "../navegacion";

const MESES = ["enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agosto", "septiembre", "octubre", "noviembre", "diciembre"];
const DIAS = ["lun", "mar", "mié", "jue", "vie", "sáb", "dom"];
const eventos = ref<Evento[]>([]);
const error = ref("");
const mensaje = ref("");
const dia = ref(5);
const hoy = new Date();
const hoyTxt = `${hoy.getFullYear()}-${String(hoy.getMonth() + 1).padStart(2, "0")}-${String(hoy.getDate()).padStart(2, "0")}`;
const anio = ref(hoy.getFullYear());
const mes = ref(hoy.getMonth()); // 0-11
const filtro = ref("");

async function cargar() {
  error.value = "";
  try { eventos.value = await api.calendario(dia.value); } catch (e) { error.value = String(e); }
}
onMounted(cargar);

const contratos = computed(() => [...new Set(eventos.value.map((e) => e.contrato))].sort());
const visibles = computed(() => eventos.value.filter((e) => !filtro.value || e.contrato === filtro.value));

// ── Qué toca ──
const prefijoMes = `${hoy.getFullYear()}-${String(hoy.getMonth() + 1).padStart(2, "0")}`;
const atrasadas = computed(() => visibles.value.filter((e) => e.tipo === "visita" && e.estado === "atrasada"));
const esteMes = computed(() => visibles.value.filter((e) => e.estado !== "hecha" && e.estado !== "atrasada" && e.fecha.startsWith(prefijoMes)));
const proximas = computed(() => {
  const limite = new Date(hoy.getFullYear(), hoy.getMonth() + 3, 1).toISOString().slice(0, 10);
  return visibles.value.filter((e) => e.tipo !== "visita" && e.fecha >= hoyTxt && e.fecha < limite);
});

// ── Calendario mensual (semanas de lunes a domingo) ──
const celdas = computed(() => {
  const primero = new Date(anio.value, mes.value, 1);
  const hueco = (primero.getDay() + 6) % 7;
  const dias = new Date(anio.value, mes.value + 1, 0).getDate();
  const r: { dia: number | null; fecha: string; eventos: Evento[] }[] = [];
  for (let i = 0; i < hueco; i++) r.push({ dia: null, fecha: "", eventos: [] });
  for (let d = 1; d <= dias; d++) {
    const f = `${anio.value}-${String(mes.value + 1).padStart(2, "0")}-${String(d).padStart(2, "0")}`;
    r.push({ dia: d, fecha: f, eventos: visibles.value.filter((e) => e.fecha === f) });
  }
  while (r.length % 7) r.push({ dia: null, fecha: "", eventos: [] });
  return r;
});
function mover(n: number) {
  const m = mes.value + n;
  anio.value += Math.floor(m / 12);
  mes.value = ((m % 12) + 12) % 12;
}

function abrir(e: Evento) {
  if (e.tipo === "visita") {
    const n = Number(/Visita (\d+)\//.exec(e.titulo)?.[1] ?? 0);
    nav.visita = { contrato: e.contrato, numero: n };
    ir("visita");
  } else {
    ir("contratos");
  }
}

async function exportar() {
  error.value = ""; mensaje.value = "";
  try {
    const ruta = await save({ defaultPath: "mantenimiento.ics", filters: [{ name: "Calendario", extensions: ["ics"] }] });
    if (!ruta) return;
    const n = await api.exportarIcs(dia.value, ruta);
    mensaje.value = `${n} citas guardadas en ${ruta}. En Outlook: Archivo → Abrir y exportar → Importar/Exportar → «Importar un archivo iCalendar (.ics)». Lo verá también en el calendario de Teams.`;
  } catch (e) { error.value = String(e); }
}
</script>

<template>
  <h1>Qué toca</h1>
  <div v-if="error" class="error">{{ error }}</div>
  <div v-if="mensaje" class="ok">{{ mensaje }}</div>

  <div class="tres">
    <div class="panel rojo">
      <h2>Visitas atrasadas ({{ atrasadas.length }})</h2>
      <p v-if="!atrasadas.length" class="pequeño">Ninguna.</p>
      <div v-for="e in atrasadas" :key="e.titulo" class="linea">
        <span><b>{{ e.titulo }}</b><br /><span class="pequeño">prevista el {{ e.fecha }}</span></span>
        <button class="primario" @click="abrir(e)">Registrar</button>
      </div>
    </div>
    <div class="panel">
      <h2>Este mes ({{ esteMes.length }})</h2>
      <p v-if="!esteMes.length" class="pequeño">Nada pendiente.</p>
      <div v-for="e in esteMes" :key="e.titulo" class="linea">
        <span><b>{{ e.titulo }}</b><br /><span class="pequeño">{{ e.fecha }} · {{ e.detalle }}</span></span>
        <button v-if="e.tipo === 'visita'" @click="abrir(e)">Registrar</button>
      </div>
    </div>
    <div class="panel">
      <h2>Inspecciones y certificados (3 meses)</h2>
      <p v-if="!proximas.length" class="pequeño">Nada en los próximos tres meses.</p>
      <div v-for="e in proximas" :key="e.titulo" class="linea">
        <span><b>{{ e.titulo }}</b><br /><span class="pequeño">{{ e.fecha }} · {{ e.detalle }}</span></span>
      </div>
    </div>
  </div>

  <div class="panel" style="margin-top: 12px">
    <div class="fila cabeza">
      <button @click="mover(-1)">‹</button>
      <h2 class="mes">{{ MESES[mes] }} {{ anio }}</h2>
      <button @click="mover(1)">›</button>
      <button @click="anio = hoy.getFullYear(); mes = hoy.getMonth()">Hoy</button>
      <select v-model="filtro"><option value="">Todos los contratos</option><option v-for="c in contratos" :key="c">{{ c }}</option></select>
      <label class="campo" title="Día del mes en que se prevén las visitas (pasa al primer laborable)">Visitas el día
        <input class="num" type="number" min="1" max="28" v-model.number="dia" @change="cargar" /></label>
      <span style="flex: 1"></span>
      <button @click="exportar">Exportar a Outlook / Teams (.ics)</button>
    </div>
    <div class="leyenda pequeño">
      <span class="ev hecha">hecha</span><span class="ev pendiente">pendiente</span><span class="ev atrasada">atrasada</span>
      <span class="ev inspeccion">inspección</span><span class="ev certificado">certificado</span>
    </div>
    <div class="mesgrid">
      <div v-for="d in DIAS" :key="d" class="dsem">{{ d }}</div>
      <div v-for="(c, i) in celdas" :key="i" class="celda" :class="{ vacia: !c.dia, hoy: c.fecha === hoyTxt }">
        <div class="num-dia">{{ c.dia }}</div>
        <div v-for="e in c.eventos" :key="e.titulo" class="ev" :class="e.tipo === 'visita' ? e.estado : e.tipo"
          :title="e.titulo + ' — ' + e.detalle" @click="abrir(e)">{{ e.titulo.replace(/ · .*/, "") }} · {{ e.contrato }}</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.tres { display: grid; grid-template-columns: repeat(3, 1fr); gap: 12px; }
.panel.rojo { border-left: 4px solid #c0392b; }
.linea { display: flex; justify-content: space-between; align-items: center; gap: 8px; padding: 6px 0; border-bottom: 1px solid var(--borde); }
.cabeza { align-items: center; gap: 8px; }
.mes { margin: 0 6px; min-width: 170px; text-align: center; text-transform: capitalize; }
.mesgrid { display: grid; grid-template-columns: repeat(7, 1fr); gap: 4px; margin-top: 8px; }
.dsem { font-size: 11px; font-weight: 700; color: var(--gris); text-align: center; text-transform: uppercase; }
.celda { min-height: 86px; border: 1px solid var(--borde); border-radius: 6px; padding: 4px; background: white; overflow: hidden; }
.celda.vacia { background: transparent; border: none; }
.celda.hoy { border: 2px solid var(--azul); }
.num-dia { font-size: 11px; color: var(--gris); }
.ev { font-size: 11px; border-radius: 4px; padding: 2px 5px; margin-top: 3px; cursor: pointer; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.ev.hecha { background: #e3f4e6; color: #1e6b2f; }
.ev.pendiente { background: #e8f0fb; color: #1f3a5f; }
.ev.atrasada { background: #fbe4e1; color: #a52a1c; font-weight: 600; }
.ev.inspeccion { background: #fff3d6; color: #7a5600; }
.ev.certificado { background: #efe6fb; color: #5b2f91; }
.leyenda { display: flex; gap: 8px; margin-top: 6px; }
.leyenda .ev { cursor: default; }
</style>
