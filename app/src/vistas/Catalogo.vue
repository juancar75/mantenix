<!--
  Catálogo y tiempos estándar.

  Los tiempos son el dato que más influye en el precio. Cada cambio se guarda al salir del campo
  y queda en el historial con el usuario y el valor anterior. La columna «Fuente» muestra de dónde
  sale la operación normativa y si está cotejada con el texto oficial.
-->
<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { api, type EquipoVista, type TiempoVista } from "../api";

const equipos = ref<EquipoVista[]>([]);
const filtro = ref("");
const familia = ref("");
const error = ref("");

onMounted(async () => { equipos.value = await api.catalogo(); });

const visibles = computed(() => equipos.value.filter((e) =>
  (!familia.value || e.familia === familia.value) &&
  (`${e.codigo} ${e.descripcion}`.toLowerCase().includes(filtro.value.toLowerCase()))));

async function guardar(e: EquipoVista, t: TiempoVista) {
  error.value = "";
  try { await api.fijarTiempo(e.codigo, t.operacion_id, t.minutos, t.tecnicos); t.estado = "revisado"; }
  catch (x) { error.value = String(x); }
}
</script>

<template>
  <h1>Catálogo y tiempos estándar</h1>
  <div v-if="error" class="error">{{ error }}</div>
  <div class="fila" style="margin-bottom: 12px">
    <input v-model="filtro" placeholder="Buscar equipo…" style="width: 280px" />
    <select v-model="familia"><option value="">Todas las familias</option><option>RITE</option><option>PCI</option></select>
  </div>

  <div v-for="e in visibles" :key="e.codigo" class="panel" style="margin-bottom: 10px">
    <div class="fila">
      <b>{{ e.codigo }}</b> {{ e.descripcion }}
      <span class="pequeño">· {{ e.familia }} · por {{ e.unidad }} · {{ e.modo_defecto }}{{ e.partida_subcontrata ? " (" + e.partida_subcontrata + ")" : "" }}</span>
    </div>
    <table v-if="e.tiempos.length" style="margin-top: 8px">
      <thead><tr><th>Operación</th><th>Period.</th><th class="der">Min/ud</th><th class="der">Técnicos</th><th>Estado</th><th>Fuente</th></tr></thead>
      <tbody>
        <tr v-for="t in e.tiempos" :key="t.operacion_id">
          <td>{{ t.texto }}<div class="pequeño">{{ t.operacion_id }} · {{ t.referencia }}</div></td>
          <td>{{ t.periodicidad }}</td>
          <td class="der"><input class="num" type="number" v-model.number="t.minutos" @change="guardar(e, t)" /></td>
          <td class="der"><input class="num" type="number" min="1" v-model.number="t.tecnicos" @change="guardar(e, t)" /></td>
          <td :class="t.estado === 'revisado' ? 'bueno' : 'pequeño'">{{ t.estado }}</td>
          <td class="pequeño" :title="t.fuente">{{ t.verificado === "no" ? "sin cotejar" : t.verificado }}</td>
        </tr>
      </tbody>
    </table>
    <div v-else class="pequeño" style="margin-top: 6px">Sin tiempos propios: se valora por tarifa de subcontrata.</div>
  </div>
</template>
