<!--
  Base de datos y usuario.

  * Usuario: los cambios quedan registrados a su nombre en el historial.
  * Abrir base: usar la base de la empresa (p. ej. en una carpeta compartida).
  * Crear base: nueva base con la normativa del programa y los datos de empresa de una carpeta
    (catálogo, tiempos, parámetros, tarifas, empresa.json). Los datos reales de la empresa no van
    en el repositorio público: se guardan aparte (carpeta `privado/`).
-->
<script setup lang="ts">
import { ref } from "vue";
import { open, save } from "@tauri-apps/plugin-dialog";
import { api } from "../api";

const emit = defineEmits<{ cambio: [] }>();
const usuario = ref("");
const error = ref("");
const mensaje = ref("");

async function intentar(f: () => Promise<void>) {
  error.value = ""; mensaje.value = "";
  try { await f(); emit("cambio"); } catch (e) { error.value = String(e); }
}

const fijarUsuario = () => intentar(async () => {
  if (!usuario.value.trim()) throw "Indique un nombre de usuario.";
  await api.fijarUsuario(usuario.value.trim());
  mensaje.value = `Los cambios se registrarán a nombre de «${usuario.value.trim()}».`;
});

const abrirBase = () => intentar(async () => {
  const ruta = await open({ filters: [{ name: "Base Mantenix", extensions: ["db"] }] });
  if (typeof ruta !== "string") return;
  await api.abrirBase(ruta);
  mensaje.value = `Base abierta: ${ruta}`;
});

const crearBase = () => intentar(async () => {
  const carpeta = await open({ directory: true, title: "Carpeta con los datos de empresa" });
  if (typeof carpeta !== "string") return;
  const ruta = await save({ defaultPath: "mantenix.db", filters: [{ name: "Base Mantenix", extensions: ["db"] }] });
  if (!ruta) return;
  await api.crearBase(ruta, carpeta);
  mensaje.value = `Base creada en ${ruta} con los datos de ${carpeta}`;
});
</script>

<template>
  <h1>Base de datos y usuario</h1>
  <div v-if="error" class="error">{{ error }}</div>
  <div v-if="mensaje" class="aviso">{{ mensaje }}</div>

  <div class="panel" style="max-width: 640px">
    <h2>Usuario</h2>
    <div class="fila"><input v-model="usuario" placeholder="Nombre o iniciales" /><button class="primario" @click="fijarUsuario">Identificarme</button></div>
    <p class="pequeño">Todos los cambios de datos se guardan en el historial con este nombre.</p>

    <h2>Base de datos</h2>
    <div class="fila"><button @click="abrirBase">Abrir base existente…</button><button @click="crearBase">Crear base con datos de empresa…</button></div>
    <p class="pequeño">La base inicial lleva datos de ejemplo. Para trabajar con vuestros datos, cree una base a partir de la carpeta con el catálogo, los tiempos, los parámetros, las tarifas y empresa.json.</p>
  </div>
</template>
