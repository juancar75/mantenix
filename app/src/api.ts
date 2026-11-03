// Llamadas al núcleo en Rust (app/src-tauri/src/main.rs).
//
// Cada función corresponde a un `#[tauri::command]`. Los tipos reflejan las estructuras de
// `mantenix-core`; si se cambia una estructura en Rust, hay que actualizarla aquí.

import { invoke } from "@tauri-apps/api/core";

// ── Tipos del núcleo ──────────────────────────────────────────────

export interface LineaEquipo { codigo: string; cantidad: number; descripcion: string; modo: string | null }
export interface Instalacion { familia: string; titulo: string; potencia_kw: number | null; temporada_meses: number; equipos: LineaEquipo[] }
export interface Contrato {
  nombre: string; anio: number; uso: string; visitas_contratadas: number | null;
  desplazamiento: { km_ida: number; minutos_ida: number };
  instalaciones: Instalacion[]; margen_pct: number | null; precio_actual_anual: number | null;
  avisos?: AvisosContrato;
  /** Datos del documento: cliente, edificio, referencia, firmantes, condiciones. */
  partes?: Record<string, any> | null;
}
export interface AvisosContrato { modalidad: string; bolsa: number | null; antiguedad: string; servicio_24h: boolean; horas_por_aviso: number }

export interface OperacionValorada { operacion_id: string; texto: string; periodicidad: string; veces_anio: number; minutos_ud: number; tecnicos: number; horas_anio: number }
export interface LineaValorada {
  instalacion: string; codigo: string; descripcion: string; cantidad: number; modo: string;
  horas_anio: number; mano_obra: number; consumibles: number; subcontrata_visita: number; operaciones: OperacionValorada[];
}
export interface Totales {
  horas_equipos: number; horas_desplazamiento: number; mano_obra: number; consumibles: number; desplazamiento: number;
  subcontrata: number; avisos: number; directo: number; indirectos: number; contingencia: number; coste_total: number; margen_pct: number;
  precio_anual: number; precio_visita: number; precio_actual_anual: number | null; margen_precio_actual_pct: number | null;
}
export interface Valoracion {
  nombre: string; anio: number; visitas: number; visitas_minimas: number; visitas_subcontrata: number;
  regimenes: { instalacion: string; descripcion: string; visitas_minimas: number }[];
  lineas: LineaValorada[]; subcontrata_visita_suma: number; subcontrata_visita_facturada: number;
  operaciones_aparte: { sistema: string; operacion_id: string; periodicidad: string; texto: string }[];
  totales: Totales; avisos: string[];
  avisos_averia: { modalidad: string; estimados: number; incluidos: number; coste_aviso: number; coste_anual: number };
}

export interface TiempoVista { operacion_id: string; texto: string; periodicidad: string; referencia: string; fuente: string; verificado: string; minutos: number; tecnicos: number; estado: string }
export interface EquipoVista { codigo: string; familia: string; descripcion: string; unidad: string; modo_defecto: string; partida_subcontrata: string; tiempos: TiempoVista[] }
export interface FilaTarifa { codigo: string; familia: string; descripcion: string; unidad: string; origen: string; minutos_revision: number; coste_revision: number; precio_revision: number; coste_anual_regimen: number; precio_anual_regimen: number }
export interface Tarifa { anio: number; coste_hora_oficial: number; coste_hora_ayudante: number; recargo_pct: number; margen_pct: number; filas: FilaTarifa[] }
export interface Cambio { id: number; fecha: string; usuario: string | null; tabla: string; accion: string; antes: string | null; despues: string | null }
export interface LineaPresupuesto {
  tipo: string; descripcion: string; unidad: string; cantidad: number; precio_unitario: number; coste_unitario: number | null;
  /** Solo en la interfaz: el precio se recalcula desde el coste (material). No se guarda. */
  auto?: boolean;
}
export interface Presupuesto {
  numero: string; fecha: string; ambito: string; tipo: string; cliente: string; cif: string; direccion: string; contacto: string; contrato: string;
  titulo: string; equipo: string; deteccion: string; averia: string; consecuencias: string; propuesta: string;
  lineas: LineaPresupuesto[]; descuento_pct: number; iva_pct: number; validez_dias: number; condiciones: string;
  estado: string; notas_internas: string;
}
export interface TotalesPresupuesto {
  lineas: { importe: number; coste: number | null }[]; subtotal: number; descuento: number; base: number; iva: number;
  total: number; coste: number; lineas_sin_coste: number; margen: number; margen_pct: number;
}
export interface ResumenPresupuesto {
  numero: string; fecha: string; cliente: string; titulo: string; estado: string; tipo: string; ambito: string; contrato: string;
  direccion: string; instalacion: string; equipo: string; base: number; total: number;
}
export interface PrecioDefecto { tipo: string; descripcion: string; unidad: string; precio_unitario: number | null; coste_unitario: number | null }

export interface ModeloPresupuesto {
  codigo: string; grupo: string; nombre: string; origen: string; ambito: string; tipo: string; titulo: string;
  equipo: string; averia: string; consecuencias: string; propuesta: string; lineas: LineaPresupuesto[];
}
export interface Ambito { codigo: string; nombre: string; habilitaciones: [string, string][] }
/** Logo guardado en los datos de empresa (imagen en base64). */
export interface Logo { tipo: string; datos: string }

export interface Estado { ruta: string; usuario: string | null; contratos: number; empresa: string }

// ── Comandos ──────────────────────────────────────────────────────

export const api = {
  estado: () => invoke<Estado>("estado"),
  fijarUsuario: (usuario: string) => invoke<void>("fijar_usuario", { usuario }),
  abrirBase: (ruta: string) => invoke<void>("abrir_base", { ruta }),
  crearBase: (ruta: string, carpetaEmpresa: string) => invoke<void>("crear_base", { ruta, carpetaEmpresa }),

  listarContratos: () => invoke<{ nombre: string; anio: number; precio: number | null }[]>("listar_contratos"),
  cargarContrato: (nombre: string) => invoke<Contrato>("cargar_contrato", { nombre }),
  guardarContrato: (contrato: Contrato) => invoke<string>("guardar_contrato", { contrato }),
  borrarContrato: (nombre: string) => invoke<void>("borrar_contrato", { nombre }),
  valorar: (nombre: string) => invoke<Valoracion>("valorar_contrato", { nombre }),
  pdfHojaCostes: (nombre: string, ruta: string) => invoke<void>("pdf_hoja_costes", { nombre, ruta }),
  pdfContrato: (nombre: string, ruta: string) => invoke<void>("pdf_contrato", { nombre, ruta }),

  listarPresupuestos: () => invoke<ResumenPresupuesto[]>("listar_presupuestos"),
  cargarPresupuesto: (numero: string) => invoke<Presupuesto>("cargar_presupuesto", { numero }),
  guardarPresupuesto: (presupuesto: Presupuesto) => invoke<string>("guardar_presupuesto", { presupuesto }),
  borrarPresupuesto: (numero: string) => invoke<void>("borrar_presupuesto", { numero }),
  calcularPresupuesto: (presupuesto: Presupuesto) => invoke<TotalesPresupuesto>("calcular_presupuesto", { presupuesto }),
  preciosPresupuesto: (anio: number) =>
    invoke<{ filas: PrecioDefecto[]; recargo_material_pct: number | null }>("precios_presupuesto", { anio }),
  pdfPresupuesto: (numero: string, ruta: string) => invoke<void>("pdf_presupuesto", { numero, ruta }),
  pdfRelacionPresupuestos: (numeros: string[], titulo: string, ruta: string) =>
    invoke<void>("pdf_relacion_presupuestos", { numeros, titulo, ruta }),

  listarModelos: () => invoke<ModeloPresupuesto[]>("listar_modelos"),
  presupuestoDesdeModelo: (codigo: string, fecha: string) => invoke<Presupuesto>("presupuesto_desde_modelo", { codigo, fecha }),
  guardarComoModelo: (presupuesto: Presupuesto, codigo: string, grupo: string, nombre: string) =>
    invoke<void>("guardar_como_modelo", { presupuesto, codigo, grupo, nombre }),
  borrarModelo: (codigo: string) => invoke<void>("borrar_modelo", { codigo }),

  empresa: () => invoke<Record<string, any>>("empresa"),
  fijarEmpresa: (datos: Record<string, any>) => invoke<void>("fijar_empresa", { datos }),
  fijarLogo: (ambito: string, ruta: string) => invoke<void>("fijar_logo", { ambito, ruta }),
  ambitos: () => invoke<Ambito[]>("ambitos"),

  catalogo: () => invoke<EquipoVista[]>("catalogo"),
  fijarTiempo: (equipo: string, operacion: string, minutos: number, tecnicos: number) =>
    invoke<void>("fijar_tiempo", { equipo, operacion, minutos, tecnicos }),

  parametros: (anio: number) => invoke<[string, number][]>("parametros", { anio }),
  fijarParametro: (anio: number, nombre: string, valor: number) => invoke<void>("fijar_parametro", { anio, nombre, valor }),
  tarifa: (anio: number) => invoke<Tarifa>("calcular_tarifa", { anio }),
  pdfTarifa: (anio: number, ruta: string) => invoke<void>("pdf_tarifa", { anio, ruta }),

  historial: (tabla: string | null, limite: number) => invoke<Cambio[]>("historial", { tabla, limite }),
};

// ── Formato ───────────────────────────────────────────────────────

const fmt = new Intl.NumberFormat("es-ES", { minimumFractionDigits: 2, maximumFractionDigits: 2, useGrouping: "always" } as unknown as Intl.NumberFormatOptions);
/** Importe en euros con formato español. */
export const eur = (x: number | null | undefined) => (x == null ? "—" : fmt.format(x) + " €");
/** Número con los decimales indicados. */
export const num = (x: number, d = 1) => new Intl.NumberFormat("es-ES", { maximumFractionDigits: d }).format(x);
