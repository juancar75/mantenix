// Hoja de costes interna de un contrato (salida del motor de valoración).
#import "comun.typ": *

// Entradas: datos (JSON del motor) y empresa (JSON con razón social).
#let empresa = json(bytes(sys.inputs.empresa))

#let val = json(bytes(sys.inputs.datos))
#let t = val.totales
#let p = val.parametros

#show: estilo.with(empresa: empresa.razon_social, "Hoja de costes", subtitulo: val.nombre + " · año " + str(val.anio))

#grid(columns: (1fr, 1fr, 1fr, 1fr), column-gutter: 6pt,
  tarjeta("Precio anual", eur(t.precio_anual), destacado: true),
  tarjeta("Precio por visita", eur(t.precio_visita)),
  tarjeta("Coste total", eur(t.coste_total)),
  tarjeta("Visitas / año", str(val.visitas) + " (mín. " + str(val.visitas_minimas) + ")"),
)

#if t.precio_actual_anual != none {
  v(6pt)
  let m = t.margen_precio_actual_pct
  block(width: 100%, inset: 8pt, radius: 3pt,
    fill: if m < p.margen_pct { rgb("#fdecea") } else { rgb("#e8f4ea") },
    [*Precio actual:* #eur(t.precio_actual_anual) → margen real *#num(m, d: 1) %* (objetivo #num(p.margen_pct, d: 0) %).
     Diferencia con el precio calculado: #eur(t.precio_anual - t.precio_actual_anual).])
}

= Régimen normativo
#for r in val.regimenes [- *#r.instalacion*: #r.descripcion — mínimo #r.visitas_minimas visitas/año.]

= Desglose del coste anual
#table(columns: (1fr, auto, auto), align: (left, right, right),
  table.header[Concepto][Horas][Importe],
  [Mano de obra en equipos], num(t.horas_equipos, d: 1) + " h", eur(t.mano_obra),
  [Desplazamiento y tiempo fijo por visita], num(t.horas_desplazamiento, d: 1) + " h", eur(t.desplazamiento),
  [Consumibles], [], eur(t.consumibles),
  [Subcontrata (#eur(val.subcontrata_visita_facturada)/visita × #val.visitas_subcontrata)], [], eur(t.subcontrata),
  [*Coste directo*], [], [*#eur(t.directo)*],
  [Indirectos (#num(p.indirectos_pct, d: 0) %)], [], eur(t.indirectos),
  [Contingencia (#num(p.contingencia_pct, d: 0) %)], [], eur(t.contingencia),
  [*Coste total*], [], [*#eur(t.coste_total)*],
  [Margen sobre venta (#num(t.margen_pct, d: 0) %)], [], eur(t.precio_anual - t.coste_total),
  table.cell(fill: fondo)[*Precio de venta anual*], table.cell(fill: fondo)[], table.cell(fill: fondo)[*#eur(t.precio_anual)*],
)
#if val.subcontrata_visita_suma > 0 and val.subcontrata_visita_suma < val.subcontrata_visita_facturada [
  #text(8pt, fill: gris)[La suma de subcontrata por visita (#eur(val.subcontrata_visita_suma)) no llega al mínimo de #eur(p.minimo_visita_subcontrata): se aplica el mínimo.]
]

= Equipos
#table(columns: (auto, 1fr, auto, auto, auto, auto, auto), align: (left, left, right, left, right, right, right),
  table.header[Código][Equipo][Cant.][Modo][Horas/año][Mano de obra][Subcontr./visita],
  ..val.lineas.map(l => (
    text(7.5pt, l.codigo), l.descripcion, num(l.cantidad, d: 0), l.modo,
    if l.modo == "propio" { num(l.horas_anio, d: 1) } else { "—" },
    if l.modo == "propio" { eur(l.mano_obra) } else { "—" },
    if l.modo == "subcontrata" { eur(l.subcontrata_visita) } else { "—" },
  )).flatten(),
)

= Detalle de operaciones con medios propios
#for l in val.lineas.filter(l => l.operaciones.len() > 0) [
  #block(sticky: true, above: 0.8em, below: 0.3em, text(weight: "bold")[#l.descripcion (× #num(l.cantidad, d: 0))])
  #table(columns: (auto, 1fr, auto, auto, auto, auto), align: (left, left, center, right, right, right),
    table.header[Operación][Descripción][Period.][Veces/año][Min/ud][Horas/año],
    ..l.operaciones.map(o => (
      text(7pt, o.operacion_id), text(8pt, o.texto), o.periodicidad, num(o.veces_anio, d: 0),
      num(o.minutos_ud, d: 0) + (if o.tecnicos > 1 { " ×" + str(o.tecnicos) } else { "" }), num(o.horas_anio, d: 1),
    )).flatten(),
  )
]

#if val.operaciones_aparte.len() > 0 [
  = Fuera de la cuota (ofertar aparte según fechas del inventario)
  #table(columns: (auto, auto, 1fr), table.header[Sistema][Period.][Operación],
    ..val.operaciones_aparte.map(o => (o.sistema, o.periodicidad, text(8pt, o.texto))).flatten())
]

#if val.avisos.len() > 0 [
  = Avisos
  #for a in val.avisos [- #a]
]

= Parámetros #val.anio
#table(columns: (1fr, auto), align: (left, right),
  table.header[Parámetro][Valor],
  [Coste hora oficial], eur(p.coste_hora_oficial),
  [Coste hora ayudante], eur(p.coste_hora_ayudante),
  [Coste por km], eur(p.coste_km),
  [Tiempo fijo por visita], num(p.tiempo_fijo_visita_min, d: 0) + " min",
  [Mínimo por visita de subcontrata], eur(p.minimo_visita_subcontrata),
)
