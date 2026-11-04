<!--
  Ventana principal: menú lateral y vista activa.
  Cada vista está en src/vistas/ y solo habla con el núcleo a través de src/api.ts.
-->
<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { nav } from "./navegacion";
import { getVersion } from "@tauri-apps/api/app";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { api, type Estado } from "./api";
import Inicio from "./vistas/Inicio.vue";
import Visita from "./vistas/Visita.vue";
import Asistente from "./vistas/Asistente.vue";
import Contratos from "./vistas/Contratos.vue";
import Presupuestos from "./vistas/Presupuestos.vue";
import Catalogo from "./vistas/Catalogo.vue";
import Parametros from "./vistas/Parametros.vue";
import TarifaVista from "./vistas/Tarifa.vue";
import Historial from "./vistas/Historial.vue";
import BaseDatos from "./vistas/BaseDatos.vue";
import Empresa from "./vistas/Empresa.vue";

const vistas = [
  { id: "inicio", titulo: "Qué toca · calendario", comp: Inicio },
  { id: "visita", titulo: "Registrar visita (asistente)", comp: Visita },
  { id: "asistente", titulo: "Nuevo contrato (asistente)", comp: Asistente },
  { id: "contratos", titulo: "Contratos", comp: Contratos },
  { id: "presupuestos", titulo: "Presupuestos", comp: Presupuestos },
  { id: "catalogo", titulo: "Catálogo y tiempos", comp: Catalogo },
  { id: "parametros", titulo: "Parámetros por año", comp: Parametros },
  { id: "tarifa", titulo: "Tarifa propia", comp: TarifaVista },
  { id: "historial", titulo: "Historial de cambios", comp: Historial },
  { id: "empresa", titulo: "Empresa y logos", comp: Empresa },
  { id: "base", titulo: "Base de datos", comp: BaseDatos },
];
const activa = computed(() => vistas.find((v) => v.id === nav.vista) ?? vistas[0]);
const estado = ref<Estado | null>(null);

async function refrescar() { estado.value = await api.estado(); }

// ── Actualizaciones: al arrancar se mira en GitHub si hay versión nueva y se pregunta ──
const version = ref("");
const nueva = ref<Update | null>(null);
const actualizando = ref("");
async function buscarActualizacion(avisarSiNoHay = false) {
  try {
    const u = await check();
    nueva.value = u;
    if (!u && avisarSiNoHay) actualizando.value = "Tiene la última versión.";
  } catch (e) {
    // Sin conexión o GitHub no responde: no molesta al arrancar.
    if (avisarSiNoHay) actualizando.value = `No se pudo comprobar: ${e}`;
  }
}
async function instalar() {
  if (!nueva.value) return;
  try {
    let total = 0, bajado = 0;
    actualizando.value = "Descargando…";
    await nueva.value.downloadAndInstall((ev) => {
      if (ev.event === "Started") total = ev.data.contentLength ?? 0;
      if (ev.event === "Progress") {
        bajado += ev.data.chunkLength;
        if (total) actualizando.value = `Descargando… ${Math.round((bajado / total) * 100)} %`;
      }
      if (ev.event === "Finished") actualizando.value = "Instalando…";
    });
    await relaunch();
  } catch (e) { actualizando.value = `No se pudo instalar: ${e}`; }
}

onMounted(async () => {
  refrescar();
  version.value = await getVersion().catch(() => "");
  buscarActualizacion();
});
</script>

<template>
  <div class="marco">
    <nav>
      <div class="marca">Mantenix</div>
      <a v-for="v in vistas" :key="v.id" :class="{ activo: activa.id === v.id }" @click="nav.vista = v.id">{{ v.titulo }}</a>
      <div class="pie" v-if="estado">
        <div>{{ estado.empresa || "Sin empresa" }}</div>
        <div>Usuario: <b>{{ estado.usuario || "sin identificar" }}</b></div>
        <div class="ruta" :title="estado.ruta">{{ estado.ruta }}</div>
        <div>Versión {{ version }} · <a class="buscar" @click="buscarActualizacion(true)">buscar actualización</a></div>
        <div v-if="actualizando && !nueva">{{ actualizando }}</div>
      </div>
    </nav>
    <main>
      <div v-if="nueva" class="actualizacion">
        <div>
          <b>Hay una versión nueva de Mantenix: {{ nueva.version }}</b> (tiene la {{ version }}).
          <div v-if="nueva.body" class="notas">{{ nueva.body }}</div>
          <div class="pequeño">Su base de datos no se toca. Antes de instalar, el programa se cierra y se vuelve a abrir solo.</div>
        </div>
        <div class="botones">
          <button class="primario" :disabled="!!actualizando" @click="instalar">{{ actualizando || "Instalar ahora" }}</button>
          <button :disabled="!!actualizando" @click="nueva = null">Más tarde</button>
        </div>
      </div>
      <component :is="activa.comp" :key="activa.id" @cambio="refrescar" />
    </main>
  </div>
</template>

<style scoped>
.marco { display: grid; grid-template-columns: 220px 1fr; height: 100vh; }
nav { background: var(--azul); color: white; display: flex; flex-direction: column; padding: 14px 0; }
.marca { font-size: 20px; font-weight: 800; padding: 0 18px 16px; }
nav a { padding: 9px 18px; cursor: pointer; color: #dfe7f1; }
nav a:hover { background: #2b4b74; }
nav a.activo { background: white; color: var(--azul); font-weight: 600; }
.pie { margin-top: auto; padding: 12px 18px; font-size: 11px; color: #b9c7d8; display: flex; flex-direction: column; gap: 3px; }
.ruta { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
main { overflow: auto; padding: 20px 24px; }
.actualizacion { display: flex; gap: 16px; align-items: center; justify-content: space-between; background: #e8f1ff; border: 1px solid #9cbcf0; border-radius: 8px; padding: 10px 14px; margin-bottom: 14px; }
.actualizacion .notas { white-space: pre-line; font-size: 12px; margin: 4px 0; max-height: 120px; overflow: auto; }
.actualizacion .botones { display: flex; gap: 8px; flex-shrink: 0; }
nav .pie a.buscar { padding: 0; color: #dfe7f1; text-decoration: underline; cursor: pointer; display: inline; }
</style>
