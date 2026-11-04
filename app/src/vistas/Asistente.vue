<!--
  Asistente de nuevo contrato: guía paso a paso a quien toma los datos.

  Pasos: inicio → cliente → edificio → climatización (RITE) → protección contra incendios →
  garaje y emergencias → avisos y condiciones → resultado (precio, hoja de costes y contrato PDF).

  Cada paso explica qué dato se pide y dónde encontrarlo. Al final se guarda como un contrato
  normal (se puede seguir editando en «Contratos») con los datos del documento.
-->
<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import { api, eur, num, type Contrato, type EquipoVista, type InspeccionPeriodica, type Valoracion } from "../api";

const emit = defineEmits<{ cambio: [] }>();

// ── Grupos de equipos que se preguntan en cada paso ──────────────────────────
const GRUPOS_RITE = [
  { titulo: "Producción de calor y frío", ayuda: "Calderas, bombas de calor, enfriadoras y unidades exteriores. Se cuentan en la sala de calderas, en cubierta o en los patios.",
    codigos: ["CALD-GAS", "CALD-GASOLEO", "CALD-MURAL", "CALD-BIO", "ENFRIADORA", "BC-GEO", "ROOFTOP", "VRF-EXT", "SPLIT-EXT", "TORRE"] },
  { titulo: "Bombas y circuitos", ayuda: "Bombas circuladoras (cada bomba cuenta, también las de reserva; una bomba doble son dos), vasos de expansión, válvulas de tres vías o de dos vías motorizadas, depósitos de inercia, intercambiadores de placas y tratamientos de agua. Circuitos: cuente uno por cada circuito (primario, cada secundario de calefacción, ACS).",
    codigos: ["BOMBA", "VASO-EXP", "VALV-REG", "DEP-INERCIA", "INTERC-PL", "TRAT-AGUA", "FILTRO-AGUA", "CIRC-HIDR"] },
  { titulo: "Emisores de calor", ayuda: "Radiadores y suelo radiante de zonas comunes o del edificio (no los de dentro de las viviendas si no entran en el contrato). El suelo radiante se cuenta por colector.",
    codigos: ["EMISOR", "SUELO-RAD"] },
  { titulo: "Equipos interiores y aire", ayuda: "Climatizadores (UTA), recuperadores, fancoils, unidades interiores, ventiladores de climatización, cortinas de aire, deshumectadoras y humidificadores. Si hay muchos, cuente por plantas.",
    codigos: ["UTA", "RECUP", "FANCOIL", "VRF-INT", "VRF-BS", "SPLIT-INT", "VENT", "CORTINA", "DESHUM", "HUMID", "PURIF", "DIFUSOR", "CONDUCTOS"] },
  { titulo: "Agua caliente sanitaria y solar", ayuda: "Acumuladores o interacumuladores de ACS, calentadores, bombas de calor de ACS y captadores solares.",
    codigos: ["ACS-SIS", "BC-ACS", "CALENT-GAS", "SOLAR-CAMPO", "SOLAR-CAPT"] },
  { titulo: "Regulación", ayuda: "Centralitas de regulación o sistema de gestión del edificio.", codigos: ["CONTROL", "CAL-AMB"] },
];
const GRUPOS_PCI = [
  { titulo: "Extintores", ayuda: "Cuente los extintores por tipo (mire la etiqueta: polvo, CO₂, agua o espuma).", codigos: ["EXT-POLVO", "EXT-CO2", "EXT-AGUA", "EXT-CARRO"] },
  { titulo: "Detección y alarma", ayuda: "La central suele estar en conserjería o en el acceso al garaje. Los detectores están en el techo; los pulsadores, junto a las salidas.",
    codigos: ["DET-CENTRAL-CONV", "DET-CENTRAL-ANA", "DET-OPT", "DET-TERM", "DET-MULTI", "DET-PULS", "DET-SIR", "DET-SIR-OA", "DET-FA", "DET-MOD", "DET-RETEN"] },
  { titulo: "Agua contra incendios", ayuda: "Bocas de incendio (armarios con manguera), grupo de presión y aljibe, hidrantes y columna seca.",
    codigos: ["BIE-25", "BIE-45", "GP-S", "GP-M", "GP-L", "ALJIBE", "HID-EXT", "HID-COL", "CS-S", "CS-M"] },
  { titulo: "Control de humos y sobrepresión", ayuda: "Ventiladores de extracción de humos del garaje (marcados F400 o F300) y su cuadro. Cuadro de mando de bomberos: caja roja «uso exclusivo bomberos», normalmente en la entrada del garaje; cuente uno por cada cuadro aunque mande también la ventilación normal. Kits de sobrepresión de escalera: ventilador de impulsión (normalmente en cubierta o en la base de la escalera), compuerta de alivio y cuadro; cuente uno por escalera presurizada.", codigos: ["SCTEH-SIS", "SCTEH-VENT", "SCTEH-CUADRO", "SCH-BOMB", "SOBREP-KIT"] },
  { titulo: "Señalización", ayuda: "Señales fotoluminiscentes de evacuación y de medios de extinción.", codigos: ["SEN-UD"] },
];
const GRUPOS_OTROS = [
  { titulo: "Ventilación del garaje y cuartos técnicos", ayuda: "Central de detección de monóxido de carbono y sus detectores (en las paredes del garaje, a 1,5 m del suelo aprox.). Extractores de garaje que no sean de humos (si son F400/F300, van en «Control de humos» del paso anterior) y extractores de cuartos técnicos, trasteros o cuartos de basuras. Cuente también los cuadros de maniobra de esos ventiladores (el cuadro de bomberos va en el paso anterior).", codigos: ["CO-CENTRAL", "CO-DET", "VENT-GAR", "VENT-CT", "VENT-CUADRO"] },
  { titulo: "Alumbrado de emergencia", ayuda: "Luminarias de emergencia (con piloto verde encendido) en escaleras, pasillos y garaje.", codigos: ["AE-UD"] },
];

const PASOS = ["Inicio", "Cliente", "Edificio", "Climatización", "Incendios", "Garaje y emergencias", "Avisos y condiciones", "Resultado"];

// ── Estado ───────────────────────────────────────────────────────────────────
const paso = ref(0);
const catalogo = ref<EquipoVista[]>([]);
const error = ref("");
const mensaje = ref("");
const valoracion = ref<Valoracion | null>(null);
const guardado = ref<string | null>(null);
const hoy = new Date();
const MESES = ["enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agosto", "septiembre", "octubre", "noviembre", "diciembre"];

const f = reactive({
  cliente: { razon_social: "", cif: "", domicilio: "", firmante: "", cargo: "" },
  edificio: { nombre: "", direccion: "", uso: "viviendas", antiguedad: "media", km: 0, minutos: 0, aparcamiento: null as number | null },
  rite: { hay: true, potencia: null as number | null, meses: 7, cantidades: {} as Record<string, number>, descripciones: {} as Record<string, string> },
  pci: { hay: true, cantidades: {} as Record<string, number>, subcontrata: true },
  otros: { cantidades: {} as Record<string, number> },
  condiciones: {
    modalidad: "bolsa", bolsa: null as number | null, visitas: null as number | null, margen: null as number | null,
    precio_actual: null as number | null, referencia: "", lugar: "Madrid", duracion: "un año desde la fecha de firma",
  },
});

// Inspecciones reglamentarias: marcadas = se incluyen en el contrato (valoradas aparte).
const tablaInsp = ref<InspeccionPeriodica[]>([]);
const insp = reactive<Record<string, { si: boolean; ultima: number | null; precio: number | null; sugerida: boolean }>>({});
let sugeridasCargadas = false;

onMounted(async () => {
  catalogo.value = await api.catalogo();
  tablaInsp.value = await api.inspeccionesCatalogo();
  for (const i of tablaInsp.value) insp[i.id] = { si: false, ultima: null, precio: null, sugerida: false };
});

async function sugerirInspecciones() {
  if (sugeridasCargadas) return;
  try {
    const ids = await api.inspeccionesSugeridas(contrato());
    for (const id of ids) if (insp[id]) { insp[id].si = true; insp[id].sugerida = true; }
    sugeridasCargadas = true;
  } catch (e) { error.value = String(e); }
}
const desc = (codigo: string) => catalogo.value.find((e) => e.codigo === codigo)?.descripcion ?? codigo;
const total = (q: Record<string, number>) => Object.values(q).reduce((a, n) => a + (Number(n) || 0), 0);

// ── Validación de cada paso ─────────────────────────────────────────────────
const falta = computed<string[]>(() => {
  const r: string[] = [];
  if (paso.value === 1 && !f.cliente.razon_social.trim()) r.push("el nombre del cliente");
  if (paso.value === 2) {
    if (!f.edificio.nombre.trim()) r.push("el nombre del edificio");
    if (!f.edificio.direccion.trim()) r.push("la dirección");
  }
  if (paso.value === 3 && f.rite.hay) {
    if (!f.rite.potencia) r.push("la potencia térmica total");
    if (!total(f.rite.cantidades)) r.push("al menos un equipo");
  }
  if (paso.value === 4 && f.pci.hay && !total(f.pci.cantidades)) r.push("al menos un equipo");
  if (paso.value === 6 && !f.rite.hay && !f.pci.hay && !total(f.otros.cantidades)) r.push("alguna instalación (vuelva a los pasos 3 a 5)");
  return r;
});

function siguiente() {
  error.value = "";
  if (falta.value.length) { error.value = `Falta ${falta.value.join(" y ")}.`; return; }
  if (paso.value === 6) { calcular(); return; }
  paso.value++;
  if (paso.value === 6) sugerirInspecciones();
  window.scrollTo(0, 0);
}
function anterior() { error.value = ""; if (paso.value > 0) paso.value--; }

// ── Construir el contrato y valorar ─────────────────────────────────────────
function lineas(q: Record<string, number>, modo: string | null, descripciones: Record<string, string> = {}) {
  return Object.entries(q).filter(([, n]) => Number(n) > 0)
    .map(([codigo, n]) => ({ codigo, cantidad: Number(n), descripcion: descripciones[codigo]?.trim() ?? "", modo }));
}

function contrato(): Contrato {
  const instalaciones = [];
  if (f.rite.hay) instalaciones.push({ familia: "RITE", titulo: "Climatización, calefacción y agua caliente", potencia_kw: f.rite.potencia, temporada_meses: f.rite.meses, equipos: lineas(f.rite.cantidades, "propio", f.rite.descripciones) });
  const pci = f.pci.hay ? lineas(f.pci.cantidades, f.pci.subcontrata ? null : "propio") : [];
  const otros = lineas(f.otros.cantidades, "propio");
  if (pci.length || otros.length) instalaciones.push({ familia: "PCI", titulo: "Protección contra incendios, garaje y emergencias", potencia_kw: null, temporada_meses: 12, equipos: [...pci, ...otros] });
  const c = f.condiciones;
  return {
    nombre: f.edificio.nombre.trim(), anio: hoy.getFullYear(),
    uso: f.edificio.uso === "viviendas" ? "viviendas" : "otros",
    visitas_contratadas: c.visitas || null,
    desplazamiento: { km_ida: Number(f.edificio.km) || 0, minutos_ida: Number(f.edificio.minutos) || 0, aparcamiento: f.edificio.aparcamiento ?? null },
    instalaciones, margen_pct: c.margen || null, precio_actual_anual: c.precio_actual || null,
    inspecciones: Object.entries(insp).filter(([, v]) => v.si).map(([id, v]) => ({ id, ultima: v.ultima || null, precio_organismo: v.precio || null })),
    avisos: { modalidad: c.modalidad, bolsa: c.modalidad === "bolsa" ? c.bolsa || null : null, antiguedad: f.edificio.antiguedad, servicio_24h: f.edificio.uso === "24h", horas_por_aviso: 1.5 },
    partes: {
      contrato: { referencia: c.referencia.trim() || `MTO-${hoy.getFullYear()}-${String(Date.now()).slice(-4)}`, lugar_fecha: `${c.lugar}, ${hoy.getDate()} de ${MESES[hoy.getMonth()]} de ${hoy.getFullYear()}` },
      cliente: { razon_social: f.cliente.razon_social.trim(), cif: f.cliente.cif.trim(), domicilio: f.cliente.domicilio.trim() || f.edificio.direccion.trim(),
        firmante: { nombre: f.cliente.firmante.trim(), dni: "", cargo: f.cliente.cargo.trim() } },
      edificio: { nombre: f.edificio.nombre.trim(), direccion: f.edificio.direccion.trim(), uso: USOS[f.edificio.uso] },
      condiciones: { duracion: c.duracion },
    },
  };
}

const USOS: Record<string, string> = { viviendas: "Residencial vivienda", oficinas: "Oficinas", comercial: "Comercial", "24h": "Hotel, residencia o servicio 24 horas" };

async function calcular() {
  error.value = "";
  try {
    const k = contrato();
    const existe = (await api.listarContratos()).some((c) => c.nombre === k.nombre);
    if (existe && guardado.value !== k.nombre && !confirm(`Ya hay un contrato llamado «${k.nombre}». ¿Sustituirlo?`)) return;
    await api.guardarContrato(k);
    guardado.value = k.nombre;
    valoracion.value = await api.valorar(k.nombre);
    paso.value = 7;
    emit("cambio");
  } catch (e) { error.value = String(e); }
}

async function pdf(tipo: "hoja" | "contrato" | "visitas" | "certificado") {
  if (!guardado.value) return;
  error.value = ""; mensaje.value = "";
  try {
    const nombres = { hoja: "hoja-costes", contrato: "contrato", visitas: "plan-y-visitas", certificado: "certificado-mantenimiento" };
    const ruta = await save({ defaultPath: `${nombres[tipo]}-${guardado.value}.pdf`, filters: [{ name: "PDF", extensions: ["pdf"] }] });
    if (!ruta) return;
    if (tipo === "hoja") await api.pdfHojaCostes(guardado.value, ruta);
    else if (tipo === "visitas") await api.pdfVisitas(guardado.value, 0, ruta);
    else if (tipo === "certificado") await api.pdfCertificado(guardado.value, hoy.getFullYear(), ruta);
    else await api.pdfContrato(guardado.value, ruta);
    mensaje.value = `PDF guardado en ${ruta}`;
  } catch (e) { error.value = String(e); }
}

function empezarOtro() {
  if (!confirm("¿Empezar un contrato nuevo? Los datos del asistente se borrarán (el contrato guardado se conserva).")) return;
  location.reload();
}
</script>

<template>
  <h1>Nuevo contrato · asistente</h1>

  <ol class="pasos">
    <li v-for="(t, i) in PASOS" :key="t" :class="{ hecho: i < paso, actual: i === paso }">
      <span class="n">{{ i < paso ? "✓" : i + 1 }}</span>{{ t }}
    </li>
  </ol>

  <div v-if="error" class="error">{{ error }}</div>
  <div v-if="mensaje" class="aviso">{{ mensaje }}</div>

  <!-- 0. Inicio -->
  <div v-if="paso === 0" class="panel paso">
    <h2>Antes de empezar</h2>
    <p>Este asistente le guiará para dar de alta un contrato de mantenimiento. Al final verá el precio y podrá sacar la
      <b>hoja de costes</b> (interna) y el <b>contrato en PDF</b> para firmar.</p>
    <p>Conviene tener a mano:</p>
    <ul>
      <li>Datos del cliente: nombre o razón social, CIF y quién firma (presidente, administrador…).</li>
      <li>Dirección del edificio y distancia desde nuestra base.</li>
      <li>La lista de equipos: mejor si ha visitado el edificio o tiene el proyecto o un contrato anterior.</li>
      <li>La potencia de las calderas o bombas de calor (está en la placa de características del equipo).</li>
    </ul>
    <p class="pequeño">Si no conoce un dato, déjelo vacío o en 0 y continúe: podrá corregirlo después en «Contratos».</p>
  </div>

  <!-- 1. Cliente -->
  <div v-if="paso === 1" class="panel paso">
    <h2>¿Quién contrata?</h2>
    <p class="ayuda">Es quien firma y paga: la comunidad de propietarios, la empresa o el particular titular de la instalación.</p>
    <label class="campo">Nombre o razón social *<input v-model="f.cliente.razon_social" placeholder="Comunidad de Propietarios Calle Mayor 5" /></label>
    <div class="fila">
      <label class="campo">CIF / NIF<input v-model="f.cliente.cif" placeholder="H-12345678" /></label>
      <label class="campo" style="flex: 1">Domicilio del cliente (si es distinto del edificio)<input v-model="f.cliente.domicilio" /></label>
    </div>
    <div class="fila">
      <label class="campo" style="flex: 1">Persona que firma<input v-model="f.cliente.firmante" placeholder="Nombre y apellidos" /></label>
      <label class="campo" style="flex: 1">Cargo<input v-model="f.cliente.cargo" placeholder="Presidente de la comunidad, administrador…" /></label>
    </div>
  </div>

  <!-- 2. Edificio -->
  <div v-if="paso === 2" class="panel paso">
    <h2>¿Dónde está la instalación?</h2>
    <label class="campo">Nombre del edificio * <span class="pequeño">(así aparecerá en la lista de contratos)</span>
      <input v-model="f.edificio.nombre" placeholder="Edificio Calle Mayor 5" /></label>
    <label class="campo">Dirección *<input v-model="f.edificio.direccion" placeholder="Calle Mayor 5, 28001 Madrid" /></label>
    <div class="opciones">
      <span class="etiqueta">Uso del edificio</span>
      <label v-for="(t, k) in USOS" :key="k" :class="{ sel: f.edificio.uso === k }"><input type="radio" v-model="f.edificio.uso" :value="k" />{{ t }}</label>
    </div>
    <p class="ayuda">El uso cambia la frecuencia de revisión de los equipos pequeños (RITE) y el número de avisos esperado.</p>
    <div class="opciones">
      <span class="etiqueta">Antigüedad de las instalaciones</span>
      <label :class="{ sel: f.edificio.antiguedad === 'nueva' }"><input type="radio" v-model="f.edificio.antiguedad" value="nueva" />Menos de 5 años</label>
      <label :class="{ sel: f.edificio.antiguedad === 'media' }"><input type="radio" v-model="f.edificio.antiguedad" value="media" />De 5 a 15 años</label>
      <label :class="{ sel: f.edificio.antiguedad === 'antigua' }"><input type="radio" v-model="f.edificio.antiguedad" value="antigua" />Más de 15 años</label>
    </div>
    <div class="fila">
      <label class="campo">Distancia desde la base (km, solo ida)<input class="num" type="number" v-model.number="f.edificio.km" /></label>
      <label class="campo">Tiempo de viaje (minutos, solo ida)<input class="num" type="number" v-model.number="f.edificio.minutos" /></label>
      <label class="campo">Aparcamiento (€ por salida)<input class="num" type="number" v-model.number="f.edificio.aparcamiento" placeholder="del año" /></label>
    </div>
    <p class="ayuda">Puede mirarlo en un navegador de mapas desde la nave hasta el edificio. Aparcamiento: lo que cuesta aparcar en cada salida (zona SER u ORA, parking); vacío = el de «Parámetros por año».</p>
  </div>

  <!-- 3. Climatización -->
  <div v-if="paso === 3" class="panel paso">
    <h2>Climatización, calefacción y agua caliente (RITE)</h2>
    <div class="opciones">
      <label :class="{ sel: f.rite.hay }"><input type="radio" v-model="f.rite.hay" :value="true" />Sí, incluir</label>
      <label :class="{ sel: !f.rite.hay }"><input type="radio" v-model="f.rite.hay" :value="false" />No hay o no se incluye</label>
    </div>
    <template v-if="f.rite.hay">
      <div class="fila">
        <label class="campo">Potencia térmica total (kW) *<input class="num" type="number" v-model.number="f.rite.potencia" /></label>
        <label class="campo">Meses de funcionamiento al año
          <select v-model.number="f.rite.meses">
            <option :value="7">7 — solo calefacción (octubre a abril)</option>
            <option :value="5">5 — solo refrigeración (mayo a septiembre)</option>
            <option :value="12">12 — todo el año (calor y frío, o ACS)</option>
          </select>
        </label>
      </div>
      <p class="ayuda">Potencia: sume la potencia de placa de todas las calderas, o la de calefacción de todas las bombas de calor
        o enfriadoras (la mayor de calor o frío). Con más de 70 kW el contrato es obligatorio y lleva certificado anual.</p>
      <div v-for="g in GRUPOS_RITE" :key="g.titulo" class="grupo">
        <h3>{{ g.titulo }}</h3>
        <p class="ayuda">{{ g.ayuda }}</p>
        <div class="equipos">
          <label v-for="c in g.codigos" :key="c" :class="{ con: (f.rite.cantidades[c] ?? 0) > 0 }">
            <span>{{ desc(c) }}</span>
            <input class="num" type="number" min="0" v-model.number="f.rite.cantidades[c]" placeholder="0" />
            <input v-if="(f.rite.cantidades[c] ?? 0) > 0" v-model="f.rite.descripciones[c]" class="marca" placeholder="Marca y modelo (opcional)" />
          </label>
        </div>
      </div>
    </template>
  </div>

  <!-- 4. Incendios -->
  <div v-if="paso === 4" class="panel paso">
    <h2>Protección contra incendios (RIPCI)</h2>
    <div class="opciones">
      <label :class="{ sel: f.pci.hay }"><input type="radio" v-model="f.pci.hay" :value="true" />Sí, incluir</label>
      <label :class="{ sel: !f.pci.hay }"><input type="radio" v-model="f.pci.hay" :value="false" />No hay o no se incluye</label>
    </div>
    <template v-if="f.pci.hay">
      <div class="opciones">
        <span class="etiqueta">¿Quién hace las revisiones de incendios?</span>
        <label :class="{ sel: f.pci.subcontrata }"><input type="radio" v-model="f.pci.subcontrata" :value="true" />La subcontrata (según su tarifa)</label>
        <label :class="{ sel: !f.pci.subcontrata }"><input type="radio" v-model="f.pci.subcontrata" :value="false" />Nosotros (medios propios)</label>
      </div>
      <p class="ayuda">El control de humos se hace siempre con medios propios. Las revisiones de incendios son trimestrales y anuales (anexo II del RIPCI).</p>
      <div v-for="g in GRUPOS_PCI" :key="g.titulo" class="grupo">
        <h3>{{ g.titulo }}</h3>
        <p class="ayuda">{{ g.ayuda }}</p>
        <div class="equipos">
          <label v-for="c in g.codigos" :key="c" :class="{ con: (f.pci.cantidades[c] ?? 0) > 0 }">
            <span>{{ desc(c) }}</span>
            <input class="num" type="number" min="0" v-model.number="f.pci.cantidades[c]" placeholder="0" />
          </label>
        </div>
      </div>
    </template>
  </div>

  <!-- 5. Garaje y emergencias -->
  <div v-if="paso === 5" class="panel paso">
    <h2>Garaje y alumbrado de emergencia</h2>
    <p class="ayuda">Si el edificio no tiene garaje ni luces de emergencia, deje todo en 0 y continúe.</p>
    <div v-for="g in GRUPOS_OTROS" :key="g.titulo" class="grupo">
      <h3>{{ g.titulo }}</h3>
      <p class="ayuda">{{ g.ayuda }}</p>
      <div class="equipos">
        <label v-for="c in g.codigos" :key="c" :class="{ con: (f.otros.cantidades[c] ?? 0) > 0 }">
          <span>{{ desc(c) }}</span>
          <input class="num" type="number" min="0" v-model.number="f.otros.cantidades[c]" placeholder="0" />
        </label>
      </div>
    </div>
  </div>

  <!-- 6. Avisos y condiciones -->
  <div v-if="paso === 6" class="panel paso">
    <h2>Averías y condiciones</h2>
    <div class="opciones vertical">
      <span class="etiqueta">¿Cómo se atienden los avisos de avería?</span>
      <label :class="{ sel: f.condiciones.modalidad === 'bolsa' }"><input type="radio" v-model="f.condiciones.modalidad" value="bolsa" />
        <span><b>Avisos incluidos</b> (recomendado): el contrato dice que se atienden los avisos, sin número. El programa estima cuántos habrá y los mete en el precio; si hay más, perdemos, y si hay menos, ganamos.</span></label>
      <label :class="{ sel: f.condiciones.modalidad === 'incluidos' }"><input type="radio" v-model="f.condiciones.modalidad" value="incluidos" />
        <span><b>Todos incluidos</b>: igual que el anterior, pero el número estimado no se puede ajustar a mano.</span></label>
      <label :class="{ sel: f.condiciones.modalidad === 'aparte' }"><input type="radio" v-model="f.condiciones.modalidad" value="aparte" />
        <span><b>Aparte</b>: cada aviso se cobra por horas y salida.</span></label>
    </div>
    <label v-if="f.condiciones.modalidad === 'bolsa'" class="campo">Avisos que se valoran al año (interno, no sale en el contrato)
      <input class="num" type="number" v-model.number="f.condiciones.bolsa" placeholder="auto" />
      <span class="pequeño">Vacío = los que calcula el programa según los equipos, la antigüedad y el uso.</span></label>
    <div class="fila">
      <label class="campo">Visitas al año<input class="num" type="number" v-model.number="f.condiciones.visitas" placeholder="mínimo" />
        <span class="pequeño">Vacío = las mínimas de la normativa.</span></label>
      <label class="campo">Margen %<input class="num" type="number" v-model.number="f.condiciones.margen" placeholder="del año" />
        <span class="pequeño">Vacío = el de «Parámetros por año».</span></label>
      <label class="campo">Precio actual €/año<input class="num" type="number" v-model.number="f.condiciones.precio_actual" />
        <span class="pequeño">Solo si es una renovación o el cliente ya paga otro mantenimiento.</span></label>
    </div>
    <div class="fila">
      <label class="campo">Referencia del contrato<input v-model="f.condiciones.referencia" placeholder="automática" /></label>
      <label class="campo">Lugar de firma<input v-model="f.condiciones.lugar" /></label>
      <label class="campo" style="flex: 1">Duración<input v-model="f.condiciones.duracion" /></label>
    </div>
    <p class="ayuda">El pago se fija solo según las visitas: mensual con 12, trimestral con 4, etc.</p>

    <h2>Inspecciones reglamentarias (se cobran aparte)</h2>
    <p class="ayuda">Inspecciones de OCA/EICI, distribuidora o empresa habilitada. No entran en la cuota: el contrato dice que avisamos, acompañamos y cobramos el acompañamiento aparte. Vienen marcadas las que parecen corresponder por los equipos (<b>sugerida</b>); compruebe la condición y desmarque las que no apliquen. Si sabe el año de la última y el precio del organismo, póngalos.</p>
    <table class="insp">
      <thead><tr><th></th><th>Inspección</th><th>Cada</th><th>Última (año)</th><th>Precio organismo €</th></tr></thead>
      <tbody>
        <tr v-for="i in tablaInsp" :key="i.id" :class="{ con: insp[i.id]?.si }">
          <td><input type="checkbox" v-model="insp[i.id].si" /></td>
          <td>{{ i.texto }} <span v-if="insp[i.id]?.sugerida" class="bueno pequeño">sugerida</span>
            <div class="pequeño">{{ i.reglamento }} · {{ i.condicion }} · {{ i.agente }}</div></td>
          <td>{{ i.periodicidad_anios }} años</td>
          <td><input class="num" type="number" v-model.number="insp[i.id].ultima" :disabled="!insp[i.id]?.si" placeholder="—" /></td>
          <td><input class="num" type="number" v-model.number="insp[i.id].precio" :disabled="!insp[i.id]?.si" placeholder="según oferta" /></td>
        </tr>
      </tbody>
    </table>
  </div>

  <!-- 7. Resultado -->
  <div v-if="paso === 7 && valoracion" class="paso">
    <div class="panel">
      <h2>{{ guardado }}</h2>
      <div class="tarjetas">
        <div class="tarjeta destacada"><div class="etiqueta">Precio anual</div><div class="valor">{{ eur(valoracion.totales.precio_anual) }}</div></div>
        <div class="tarjeta"><div class="etiqueta">Por visita</div><div class="valor">{{ eur(valoracion.totales.precio_visita) }}</div></div>
        <div class="tarjeta"><div class="etiqueta">Visitas al año</div><div class="valor">{{ valoracion.visitas }}</div></div>
        <div class="tarjeta"><div class="etiqueta">Avisos incluidos</div><div class="valor">{{ valoracion.avisos_averia.incluidos }}</div></div>
      </div>
      <ul class="pequeño"><li v-for="r in valoracion.regimenes" :key="r.instalacion">{{ r.descripcion }}</li></ul>
      <div v-for="a in valoracion.avisos" :key="a" class="aviso">{{ a }}</div>
      <table style="margin-top: 8px">
        <tbody>
          <tr><td>Mano de obra ({{ num(valoracion.totales.horas_equipos) }} h)</td><td class="der">{{ eur(valoracion.totales.mano_obra) }}</td></tr>
          <tr><td>Desplazamientos</td><td class="der">{{ eur(valoracion.totales.desplazamiento) }}</td></tr>
          <tr><td>Subcontrata de incendios</td><td class="der">{{ eur(valoracion.totales.subcontrata) }}</td></tr>
          <tr><td>Avisos incluidos ({{ valoracion.avisos_averia.incluidos }} × {{ eur(valoracion.avisos_averia.coste_aviso) }})</td><td class="der">{{ eur(valoracion.totales.avisos) }}</td></tr>
          <tr><td><b>Coste total</b> (con indirectos)</td><td class="der"><b>{{ eur(valoracion.totales.coste_total) }}</b></td></tr>
          <tr><td><b>Precio de venta</b> (margen {{ valoracion.totales.margen_pct }} %)</td><td class="der"><b>{{ eur(valoracion.totales.precio_anual) }}</b></td></tr>
          <tr v-if="valoracion.totales.limpiezas_anio"><td>Limpiezas que se presupuestan aparte (orientativo, no incluido)</td><td class="der">{{ eur(valoracion.totales.limpiezas_anio) }}</td></tr>
          <tr v-for="i in valoracion.inspecciones" :key="i.id"><td>Inspección aparte: {{ i.texto }} (cada {{ i.periodicidad_anios }} años{{ i.proxima ? ", próxima " + i.proxima : "" }})</td><td class="der">{{ eur(i.precio_total) }}{{ i.precio_organismo == null ? " + organismo" : "" }}</td></tr>
        </tbody>
      </table>
    </div>
    <div class="panel" style="margin-top: 10px">
      <h2 style="margin-top: 0">Documentos</h2>
      <div class="fila">
        <button class="primario" @click="pdf('contrato')">Contrato en PDF</button>
        <button @click="pdf('hoja')">Hoja de costes (interna)</button>
        <button @click="pdf('visitas')">Plan y hojas de visita</button>
        <button v-if="f.rite.hay" @click="pdf('certificado')">Certificado anual RITE</button>
        <span style="flex: 1"></span>
        <button @click="paso = 1">Corregir datos</button>
        <button @click="empezarOtro">Otro contrato</button>
      </div>
      <p class="pequeño">Revise el contrato antes de enviarlo. Para cambiar equipos o cantidades más adelante, use «Contratos».</p>
    </div>
  </div>

  <div v-if="paso < 7" class="fila navegacion">
    <button v-if="paso > 0" @click="anterior">← Anterior</button>
    <span style="flex: 1"></span>
    <span class="pequeño">Paso {{ paso + 1 }} de {{ PASOS.length }}</span>
    <button class="primario" @click="siguiente">{{ paso === 6 ? "Calcular precio" : paso === 0 ? "Empezar →" : "Siguiente →" }}</button>
  </div>
</template>

<style scoped>
.insp td { vertical-align: top; }
.insp tr.con td { background: var(--azul-claro); }
.pasos { list-style: none; display: flex; flex-wrap: wrap; gap: 6px; padding: 0; margin: 0 0 14px; }
.pasos li { display: flex; align-items: center; gap: 6px; font-size: 12px; color: var(--gris); padding: 4px 10px 4px 4px; border-radius: 14px; background: white; border: 1px solid var(--borde); }
.pasos .n { display: inline-flex; width: 20px; height: 20px; border-radius: 50%; align-items: center; justify-content: center; background: var(--fondo); font-weight: 700; font-size: 11px; }
.pasos li.actual { border-color: var(--azul); color: var(--azul); font-weight: 600; }
.pasos li.actual .n { background: var(--azul); color: white; }
.pasos li.hecho .n { background: var(--verde); color: white; }
.paso { max-width: 980px; }
.paso h2 { margin-top: 0; }
.paso h3 { font-size: 14px; margin: 16px 0 2px; color: var(--azul); }
.paso .campo { margin-top: 10px; }
.paso .campo input:not(.num) { width: 100%; }
.ayuda { font-size: 12.5px; color: var(--gris); background: var(--fondo); border-left: 3px solid var(--borde); padding: 6px 10px; margin: 8px 0; }
.opciones { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; margin-top: 12px; }
.opciones.vertical { flex-direction: column; align-items: stretch; }
.opciones .etiqueta { width: 100%; font-size: 12px; color: var(--gris); }
.opciones label { display: flex; gap: 8px; align-items: flex-start; border: 1px solid var(--borde); border-radius: 8px; padding: 8px 12px; cursor: pointer; background: white; }
.opciones label.sel { border-color: var(--azul); background: var(--azul-claro); }
.equipos { display: grid; grid-template-columns: repeat(auto-fill, minmax(290px, 1fr)); gap: 6px; }
.equipos label { display: grid; grid-template-columns: 1fr 80px; gap: 4px 8px; align-items: center; border: 1px solid var(--borde); border-radius: 6px; padding: 6px 8px; background: white; }
.equipos label.con { border-color: var(--azul); background: var(--azul-claro); }
.equipos .marca { grid-column: 1 / -1; font-size: 12px; }
.navegacion { margin-top: 14px; max-width: 980px; }
</style>
