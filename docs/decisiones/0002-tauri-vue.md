# 0002. Aplicación de escritorio con Tauri 2 y Vue 3

- Estado: aceptada
- Fecha: 2026-10-13

## Contexto
Se necesita una aplicación de escritorio para Windows (y Linux/macOS), con tablas editables, que pueda tener más adelante una versión para tableta Android.

## Decisión
Tauri 2 con interfaz en Vue 3 + TypeScript. La lógica queda en Rust (`mantenix-core`); la interfaz solo llama a comandos.

## Alternativas descartadas
- **egui/iced (Rust puro)**: menos maduros para tablas grandes y formularios.
- **Electron**: ejecutables de más de 100 MB y sin núcleo Rust compartido.

## Consecuencias
- Instaladores pequeños; Tauri 2 permite Android en el futuro.
- Los tipos de Rust se repiten en `app/src/api.ts` y deben mantenerse sincronizados.
