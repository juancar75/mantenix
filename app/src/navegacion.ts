// Navegación entre vistas con datos: un asistente puede abrir otro ya rellenado
// (p. ej. «Registrar visita» → «Presupuestos» con el correctivo de un defecto).
import { reactive } from "vue";

export interface CorrectivoPendiente {
  contrato: string; cliente: string; direccion: string; ambito: string; equipo: string; averia: string;
}

export const nav = reactive({
  vista: "inicio",
  /** Contrato y visita que debe abrir «Registrar visita». */
  visita: null as { contrato: string; numero: number } | null,
  /** Correctivo que debe crear «Presupuestos» al abrirse. */
  correctivo: null as CorrectivoPendiente | null,
});

export function ir(vista: string) { nav.vista = vista; }
