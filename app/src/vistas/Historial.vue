<!--
  Historial de cambios (tabla `auditoria`): quién cambió qué dato, cuándo, y su valor anterior.
  Los registros los crean disparadores de SQLite; la aplicación no puede omitirlos.
-->
<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { api, type Cambio } from "../api";

const tabla = ref<string | null>(null);
const cambios = ref<Cambio[]>([]);
const TABLAS = ["parametro", "tiempo", "equipo", "tarifa_subcontrata", "operacion", "contrato", "linea_equipo", "empresa"];

async function cargar() { cambios.value = await api.historial(tabla.value, 300); }
onMounted(cargar);
watch(tabla, cargar);

/** Muestra solo los campos que cambian entre el valor anterior y el nuevo. */
function diferencias(c: Cambio): string {
  const a = c.antes ? JSON.parse(c.antes) : {};
  const d = c.despues ? JSON.parse(c.despues) : {};
  if (c.accion !== "cambio") return JSON.stringify(c.accion === "alta" ? d : a);
  return Object.keys(d).filter((k) => JSON.stringify(a[k]) !== JSON.stringify(d[k]))
    .map((k) => `${k}: ${JSON.stringify(a[k])} → ${JSON.stringify(d[k])}`).join(" · ");
}
</script>

<template>
  <h1>Historial de cambios</h1>
  <div class="fila" style="margin-bottom: 12px">
    <select v-model="tabla">
      <option :value="null">Todas las tablas</option>
      <option v-for="t in TABLAS" :key="t" :value="t">{{ t }}</option>
    </select>
  </div>
  <table>
    <thead><tr><th>#</th><th>Fecha (UTC)</th><th>Usuario</th><th>Tabla</th><th>Acción</th><th>Detalle</th></tr></thead>
    <tbody>
      <tr v-for="c in cambios" :key="c.id">
        <td class="pequeño">{{ c.id }}</td><td class="pequeño">{{ c.fecha }}</td><td>{{ c.usuario ?? "—" }}</td>
        <td>{{ c.tabla }}</td><td>{{ c.accion }}</td><td class="pequeño" style="word-break: break-word">{{ diferencias(c) }}</td>
      </tr>
    </tbody>
  </table>
</template>
