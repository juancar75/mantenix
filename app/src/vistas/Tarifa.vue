<!--
  Tarifa propia: precio por unidad y revisión, calculado con los tiempos y parámetros del año.
  No se edita a mano: para cambiarla se ajustan tiempos (Catálogo) o parámetros.
-->
<script setup lang="ts">
import { ref } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import { api, eur, type Tarifa } from "../api";

const anio = ref(new Date().getFullYear());
const tarifa = ref<Tarifa | null>(null);
const error = ref("");

async function calcular() {
  error.value = "";
  try { tarifa.value = await api.tarifa(anio.value); } catch (e) { error.value = String(e); }
}
async function pdf() {
  const ruta = await save({ defaultPath: `tarifa-${anio.value}.pdf`, filters: [{ name: "PDF", extensions: ["pdf"] }] });
  if (ruta) await api.pdfTarifa(anio.value, ruta);
}
calcular();
</script>

<template>
  <h1>Tarifa propia</h1>
  <div v-if="error" class="error">{{ error }}</div>
  <div class="fila" style="margin-bottom: 12px">
    <label class="campo">Año<input class="num" type="number" v-model.number="anio" @change="calcular" /></label>
    <button @click="pdf">Exportar PDF</button>
  </div>
  <template v-if="tarifa">
    <p class="pequeño">Coste hora oficial {{ eur(tarifa.coste_hora_oficial) }} · recargo {{ tarifa.recargo_pct }} % · margen {{ tarifa.margen_pct }} %</p>
    <table>
      <thead><tr><th>Código</th><th>Equipo</th><th>Origen</th><th class="der">Min/rev.</th><th class="der">Coste/rev.</th><th class="der">Precio/rev.</th><th class="der">Precio/año</th></tr></thead>
      <tbody>
        <tr v-for="f in tarifa.filas" :key="f.codigo">
          <td class="pequeño">{{ f.codigo }}</td><td>{{ f.descripcion }}</td><td class="pequeño">{{ f.origen }}</td>
          <td class="der">{{ f.minutos_revision || "—" }}</td><td class="der">{{ eur(f.coste_revision) }}</td>
          <td class="der"><b>{{ eur(f.precio_revision) }}</b></td><td class="der">{{ eur(f.precio_anual_regimen) }}</td>
        </tr>
      </tbody>
    </table>
  </template>
</template>
