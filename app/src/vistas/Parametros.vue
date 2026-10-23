<!--
  Parámetros económicos por año (coste hora, km, indirectos, margen…).

  Cada año tiene sus propios valores. Si un año no tiene un parámetro, el motor usa el del año
  anterior más próximo; por eso basta con dar de alta solo lo que cambia (p. ej. el coste hora 2027).
-->
<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { api } from "../api";

const NOMBRES: Record<string, string> = {
  coste_hora_oficial: "Coste hora oficial (€/h)",
  coste_hora_ayudante: "Coste hora ayudante (€/h)",
  coste_km: "Coste por km (€/km)",
  tiempo_fijo_visita_min: "Tiempo fijo por visita (min)",
  indirectos_pct: "Indirectos (%)",
  contingencia_pct: "Contingencia (%)",
  margen_pct: "Margen sobre venta (%)",
  minimo_visita_subcontrata: "Mínimo por visita de subcontrata (€)",
  precio_hora_oficial: "Presupuestos: precio hora oficial (€/h)",
  precio_hora_pareja: "Presupuestos: precio hora oficial + ayudante (€/h)",
  precio_salida: "Presupuestos: precio por salida (€)",
  coste_salida: "Presupuestos: coste por salida (€)",
  recargo_material_pct: "Presupuestos: recargo sobre coste del material (%)",
};
const anio = ref(new Date().getFullYear());
const valores = ref<Record<string, number | null>>({});
const error = ref("");

async function cargar() {
  const lista = await api.parametros(anio.value);
  valores.value = Object.fromEntries(Object.keys(NOMBRES).map((k) => [k, lista.find(([n]) => n === k)?.[1] ?? null]));
}
onMounted(cargar);
watch(anio, cargar);

async function guardar(nombre: string) {
  error.value = "";
  const v = valores.value[nombre];
  if (v == null) return;
  try { await api.fijarParametro(anio.value, nombre, v); } catch (e) { error.value = String(e); }
}
</script>

<template>
  <h1>Parámetros por año</h1>
  <div v-if="error" class="error">{{ error }}</div>
  <div class="fila" style="margin-bottom: 12px">
    <label class="campo">Año<input class="num" type="number" v-model.number="anio" /></label>
  </div>
  <div class="panel" style="max-width: 560px">
    <table>
      <tbody>
        <tr v-for="(etiqueta, k) in NOMBRES" :key="k">
          <td>{{ etiqueta }}</td>
          <td class="der">
            <input class="num" type="number" step="0.01" v-model.number="valores[k]" :placeholder="'hereda'" @change="guardar(k)" />
          </td>
        </tr>
      </tbody>
    </table>
    <p class="pequeño">Los campos vacíos heredan el valor del año anterior. Cada cambio queda en el historial.</p>
  </div>
</template>
