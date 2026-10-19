<!--
  Ventana principal: menú lateral y vista activa.
  Cada vista está en src/vistas/ y solo habla con el núcleo a través de src/api.ts.
-->
<script setup lang="ts">
import { onMounted, ref, shallowRef } from "vue";
import { api, type Estado } from "./api";
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
const activa = shallowRef(vistas[0]);
const estado = ref<Estado | null>(null);

async function refrescar() { estado.value = await api.estado(); }
onMounted(refrescar);
</script>

<template>
  <div class="marco">
    <nav>
      <div class="marca">Mantenix</div>
      <a v-for="v in vistas" :key="v.id" :class="{ activo: activa.id === v.id }" @click="activa = v">{{ v.titulo }}</a>
      <div class="pie" v-if="estado">
        <div>{{ estado.empresa || "Sin empresa" }}</div>
        <div>Usuario: <b>{{ estado.usuario || "sin identificar" }}</b></div>
        <div class="ruta" :title="estado.ruta">{{ estado.ruta }}</div>
      </div>
    </nav>
    <main>
      <component :is="activa.comp" @cambio="refrescar" />
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
</style>
