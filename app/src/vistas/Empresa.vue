<!--
  Empresa: datos que aparecen en los documentos (membrete, habilitaciones, firmante) y logos.

  Cada ámbito (RITE, PCI, baja tensión) puede tener su propio logo; si no lo tiene se usa el
  general. La habilitación que sale en la carta presupuesto es la del ámbito del presupuesto.
  Todo se guarda en la base de datos, con historial de cambios.
-->
<script setup lang="ts">
import { onMounted, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { api, type Ambito, type Logo } from "../api";

const emit = defineEmits<{ cambio: [] }>();
const e = ref<Record<string, any> | null>(null);
const ambitos = ref<Ambito[]>([]);
const error = ref("");
const mensaje = ref("");

async function cargar() {
  const d = await api.empresa();
  d.habilitaciones ??= [];
  d.firmante ??= { nombre: "", cargo: "" };
  e.value = d;
  ambitos.value = await api.ambitos();
}
onMounted(cargar);

async function intentar(f: () => Promise<void>) {
  error.value = ""; mensaje.value = "";
  try { await f(); emit("cambio"); } catch (x) { error.value = String(x); }
}

const guardar = () => intentar(async () => {
  if (!e.value) return;
  e.value.habilitaciones = e.value.habilitaciones.filter((h: string[]) => h[0]?.trim() || h[1]?.trim());
  await api.fijarEmpresa(e.value);
  await cargar();
  mensaje.value = "Datos de empresa guardados.";
});

/** Logo guardado de un ámbito ("" = general). */
function logo(codigo: string): Logo | null {
  if (!e.value) return null;
  return (codigo ? e.value.ambitos?.[codigo]?.logo : e.value.logo) ?? null;
}
const MIME: Record<string, string> = { png: "image/png", jpg: "image/jpeg", svg: "image/svg+xml" };
const src = (l: Logo) => `data:${MIME[l.tipo] ?? "image/png"};base64,${l.datos}`;

const cargarLogo = (codigo: string) => intentar(async () => {
  const ruta = await open({ filters: [{ name: "Imagen", extensions: ["png", "jpg", "jpeg", "svg"] }] });
  if (typeof ruta !== "string") return;
  await api.fijarLogo(codigo, ruta);
  await cargar();
  mensaje.value = "Logo cargado.";
});

const quitarLogo = (codigo: string) => intentar(async () => {
  await api.fijarLogo(codigo, "");
  await cargar();
});
</script>

<template>
  <h1>Empresa</h1>
  <div v-if="error" class="error">{{ error }}</div>
  <div v-if="mensaje" class="aviso">{{ mensaje }}</div>

  <template v-if="e">
    <div class="panel">
      <h2 style="margin-top: 0">Datos que aparecen en los documentos</h2>
      <div class="fila">
        <label class="campo" style="flex: 2">Razón social<input v-model="e.razon_social" /></label>
        <label class="campo">CIF<input v-model="e.cif" /></label>
        <label class="campo">Teléfono<input v-model="e.telefono" /></label>
        <label class="campo" style="flex: 1">Correo electrónico<input v-model="e.email" /></label>
      </div>
      <div class="fila" style="margin-top: 8px">
        <label class="campo" style="flex: 1">Domicilio<input v-model="e.domicilio" /></label>
      </div>
      <div class="fila" style="margin-top: 8px">
        <label class="campo" style="flex: 1">Firmante de las cartas<input v-model="e.firmante.nombre" /></label>
        <label class="campo" style="flex: 1">Cargo<input v-model="e.firmante.cargo" /></label>
        <label class="campo" style="flex: 2">Horario de servicio (contratos)<input v-model="e.horario" /></label>
      </div>

      <h2>Habilitaciones</h2>
      <p class="pequeño">La carta de cada ámbito muestra solo su habilitación: la que menciona «térmicas»/«RITE»,
        «incendios»/«RIPCI» o «baja tensión»/«REBT» en el texto.</p>
      <table>
        <thead><tr><th>Descripción</th><th style="width: 200px">Número</th><th style="width: 44px"></th></tr></thead>
        <tbody>
          <tr v-for="(h, i) in e.habilitaciones" :key="i">
            <td><input v-model="h[0]" style="width: 100%" /></td>
            <td><input v-model="h[1]" style="width: 100%" /></td>
            <td><button class="peligro" @click="e.habilitaciones.splice(i, 1)">×</button></td>
          </tr>
        </tbody>
      </table>
      <div class="fila" style="margin-top: 8px">
        <button @click="e.habilitaciones.push(['', ''])">+ Habilitación</button>
        <span style="flex: 1"></span>
        <button class="primario" @click="guardar">Guardar datos</button>
      </div>
    </div>

    <div class="panel" style="margin-top: 10px">
      <h2 style="margin-top: 0">Logos por ámbito</h2>
      <p class="pequeño">PNG, JPG o SVG de hasta 1 MB. Si un ámbito no tiene logo propio se usa el general.</p>
      <div class="logos">
        <div v-for="a in ambitos" :key="a.codigo" class="logo">
          <div class="nombre">{{ a.nombre }}</div>
          <div class="marco-logo">
            <img v-if="logo(a.codigo)" :src="src(logo(a.codigo)!)" alt="logo" />
            <span v-else-if="a.codigo && logo('')" class="pequeño">Usa el general</span>
            <span v-else class="pequeño">Sin logo</span>
          </div>
          <div class="pequeño hab">
            <div v-for="h in a.habilitaciones" :key="h[1]">{{ h[0] }}: <b>{{ h[1] }}</b></div>
          </div>
          <div class="fila">
            <button @click="cargarLogo(a.codigo)">Cargar logo…</button>
            <button v-if="logo(a.codigo)" class="peligro" @click="quitarLogo(a.codigo)">Quitar</button>
          </div>
        </div>
      </div>
    </div>
  </template>
</template>

<style scoped>
.logos { display: grid; grid-template-columns: repeat(auto-fill, minmax(240px, 1fr)); gap: 12px; }
.logo { border: 1px solid var(--borde); border-radius: 8px; padding: 10px; display: flex; flex-direction: column; gap: 8px; }
.nombre { font-weight: 600; color: var(--azul); }
.marco-logo { height: 80px; display: flex; align-items: center; justify-content: center; background: var(--fondo); border-radius: 6px; }
.marco-logo img { max-height: 70px; max-width: 100%; }
.hab { min-height: 32px; }
</style>
