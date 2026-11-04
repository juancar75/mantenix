// Plan anual de mantenimiento y hojas de visita para el técnico.
//
// Entradas (sys.inputs, texto JSON):
//   plan     → mantenix_core::plan::Plan (visitas con equipos, operaciones, medidas, contadores)
//   datos    → partes del contrato (cliente, edificio, contrato.referencia)
//   empresa  → datos de la empresa (sin logos)
//   membrete → {habilitaciones, logo} del ámbito RITE
//   solo     → número de visita a imprimir, o 0 para el plan y todas las visitas
#import "comun.typ": *

#let plan = json(bytes(sys.inputs.plan))
#let d = json(bytes(sys.inputs.datos))
#let d = if d == none { (:) } else { d }
#let empresa = json(bytes(sys.inputs.empresa))
#let membrete = json(bytes(sys.inputs.membrete))
#let solo = int(sys.inputs.solo)
// Registro de la visita (si ya se hizo): rellena la hoja con lo anotado.
#let reg = json(bytes(sys.inputs.at("registro", default: "null")))
#let res(q, o) = if reg == none { none } else {
  let r = reg.resultados.filter(x => x.operacion_id == o.operacion_id and x.equipo == q.descripcion)
  if r.len() > 0 { r.first() } else { none }
}
#let marca(r, v) = if r != none and r.resultado == v {
  box(width: 8pt, height: 8pt, stroke: 0.6pt + azul, radius: 1pt, fill: azul.lighten(80%), align(center + horizon, text(7pt, weight: "bold", fill: azul, "✕")))
} else { box(width: 8pt, height: 8pt, stroke: 0.6pt + gris, radius: 1pt) }
#let lleno(x, w) = if x != none and x != "" { box(width: w, stroke: (bottom: 0.5pt + gris), text(fill: azul, x)) } else { box(width: w, stroke: (bottom: 0.5pt + gris), height: 0.9em) }
#let e = d.at("edificio", default: (nombre: plan.contrato, direccion: ""))
#let ref = d.at("contrato", default: (:)).at("referencia", default: "")
#let razon = empresa.at("razon_social", default: "")
#let caja = box(width: 8pt, height: 8pt, stroke: 0.6pt + gris, radius: 1pt)
#let en_blanco(w) = box(width: w, stroke: (bottom: 0.5pt + gris), height: 0.9em)

#set document(title: "Plan de mantenimiento · " + e.nombre)
#set text(font: "Inter", size: 8.5pt, lang: "es", region: "es")
#set par(justify: false, leading: 0.5em)
#set table(stroke: 0.4pt + linea, inset: (x: 4pt, y: 3.5pt))
#show table.cell.where(y: 0): set text(weight: "bold", size: 7.5pt, fill: azul)
#show heading.where(level: 1): it => block(above: 1em, below: 0.5em, sticky: true,
  text(10.5pt, weight: "bold", fill: azul, it.body))
#show heading.where(level: 2): it => block(above: 0.8em, below: 0.35em, sticky: true,
  text(9pt, weight: "bold", it.body))

#let cabecera(titulo, sub) = {
  grid(columns: (auto, 1fr), column-gutter: 10pt, align: horizon,
    if membrete.logo != none { image(membrete.logo, height: 1.1cm) } else { text(10pt, weight: "bold", fill: azul, razon) },
    {
      set align(right)
      text(13pt, weight: "bold", fill: azul, titulo); linebreak()
      text(8pt, fill: gris, sub)
    })
  v(2pt); line(length: 100%, stroke: 0.6pt + azul); v(2pt)
  grid(columns: (1fr, 1fr), column-gutter: 12pt, row-gutter: 3pt,
    [*Instalación:* #e.nombre], [*Contrato:* #ref],
    [*Dirección:* #e.direccion], [*Empresa:* #razon #for h in membrete.habilitaciones [· #h.at(0): #h.at(1) ]],
  )
  v(4pt)
}

#set page(paper: "a4", margin: (x: 1.4cm, top: 1.3cm, bottom: 1.4cm),
  footer: context { set text(7pt, fill: gris)
    [#razon · #e.nombre #h(1fr) Página #counter(page).display() de #counter(page).final().first()] })

// ---------------- Plan anual ----------------
#if solo == 0 {
  set page(flipped: true)
  cabecera("Plan anual de mantenimiento " + str(plan.anio), str(plan.n_visitas) + " visitas al año")
  let n = plan.n_visitas
  let mes_de(i) = plan.visitas.at(i - 1).mes_nombre.slice(0, 3)
  table(
    columns: (3.2cm, 1fr, 0.9cm) + range(n).map(_ => 0.62cm),
    align: (left, left, center) + range(n).map(_ => center),
    table.header([Equipo], [Operación], [Per.], ..range(1, n + 1).map(i => text(6.5pt)[#i \ #mes_de(i)])),
    ..plan.filas.map(f => (text(7pt, f.equipo), text(7pt, f.texto), text(7pt, f.periodicidad),
      ..range(1, n + 1).map(i => if f.visitas.contains(i) { text(fill: azul, weight: "bold")[●] } else { "" }))).flatten(),
  )
  text(7pt, fill: gris)[m: mensual · 3m: trimestral · 2t / 6m: dos veces al año (al inicio y a mitad de temporada) · t / 1a: anual, al inicio de la temporada. Las operaciones S\* las hace el usuario. Las de periodicidad superior al año no figuran en este plan. Medidas de rendimiento de generadores según la IT 3.4 del RITE.]
}

// ---------------- Hojas de visita ----------------
#for vis in plan.visitas.filter(v => solo == 0 or v.numero == solo) {
  pagebreak(weak: true)
  set page(flipped: false)
  cabecera("Hoja de visita " + str(vis.numero) + " de " + str(plan.n_visitas),
    "Prevista en " + vis.mes_nombre + " de " + str(plan.anio) + " · " + str(calc.round(vis.horas, digits: 1)).replace(".", ",") + " h previstas en equipos")
  let r0 = if reg != none and reg.numero == vis.numero { reg } else { none }
  let g(k) = if r0 == none { none } else { r0.at(k, default: none) }
  grid(columns: (1fr, 1fr, 1fr, 1fr), column-gutter: 8pt,
    [Fecha: #lleno(g("fecha"), 2.2cm)], [Entrada: #lleno(g("entrada"), 1.4cm)], [Salida: #lleno(g("salida"), 1.4cm)], [Técnico: #lleno(g("tecnico"), 2.6cm)])
  v(2pt)
  [= Operaciones]
  text(7.5pt, fill: gris)[Marque cada operación: *B* bien · *D* defecto (anótelo abajo) · *NA* no aplica.]
  table(
    columns: (1fr, auto, auto, auto),
    align: (left, center, center, center),
    table.header([Operación], [B], [D], [NA]),
    ..vis.equipos.map(q => (
      table.cell(colspan: 4, fill: fondo, text(weight: "bold")[#q.descripcion (× #num(q.cantidad, d: 0))]),
      ..q.operaciones.map(o => {
        let r = if r0 == none { none } else { res(q, o) }
        ([#o.texto #if o.nota != "" { text(7pt, fill: gris)[ — #o.nota] } #if r != none and r.nota != "" { text(7.5pt, fill: azul)[ · #r.nota] }],
          marca(r, "B"), marca(r, "D"), marca(r, "NA"))
      }).flatten(),
    )).flatten(),
  )
  if vis.subcontrata.len() > 0 {
    [= Protección contra incendios (subcontrata)]
    [Revisión trimestral por la empresa subcontratada, conforme al anexo II del RIPCI: #vis.subcontrata.join(", "). Adjunte su acta.]
  }
  for m in vis.medidas {
    [= Medidas de generadores de #m.tipo (IT 3.4 del RITE)]
    table(
      columns: (3.2cm,) + m.magnitudes.map(_ => 1fr),
      table.header([Generador], ..m.magnitudes.map(x => text(6.5pt, x))),
      ..m.generadores.map(gen => {
        let med = if r0 == none { none } else { r0.medidas.find(x => x.generador == gen) }
        (text(7.5pt, gen), ..m.magnitudes.map(mag => if med != none and med.valores.at(mag, default: "") != "" { text(fill: azul, med.valores.at(mag)) } else { v(1.1em) }))
      }).flatten(),
    )
  }
  if vis.contadores {
    [= Lectura de contadores (IT 3.4.4)]
    let ct = if r0 == none { (:) } else { r0.contadores }
    let lect(k) = if ct.at(k, default: none) != none { text(fill: azul, str(ct.at(k)).replace(".", ",")) } else { v(1.1em) }
    table(columns: (1fr, 1fr, 1fr, 1fr, 1fr),
      table.header([Combustible], [Electricidad (kWh)], [Agua de llenado (m³)], [ACS (m³)], [Energía térmica (kWh)]),
      lect("combustible"), lect("electricidad"), lect("agua_llenado"), lect("acs"), lect("energia_termica"))
  }
  [= Defectos, reparaciones y material]
  if r0 != none and (r0.defectos != "" or r0.material != "" or r0.observaciones != "") {
    table(columns: (1fr,),
      [*Defectos y acciones propuestas:* #text(fill: azul, r0.defectos)],
      [*Material:* #text(fill: azul, r0.material)],
      [*Observaciones:* #text(fill: azul, r0.observaciones)])
  } else {
    table(columns: (1fr,), ..range(4).map(_ => v(1.1em)))
  }
  v(6pt)
  grid(columns: (1fr, 1fr), column-gutter: 20pt,
    [*Técnico de la empresa mantenedora* \ Nombre y n.º de carné: \ #v(1.2cm) Firma],
    [*Recibí del titular o su representante* \ Nombre: \ #v(1.2cm) Firma])
}
