// Mantenix — Plantilla de contrato de mantenimiento PCI / ventilación
//
// Entradas (sys.inputs, texto JSON), que prepara el crate mantenix-informes:
//   datos       → datos del contrato: partes, edificio, sistemas e inventario, condiciones
//   empresa     → datos comunes de la empresa mantenedora (razón social, habilitaciones, horario)
//   operaciones → operaciones del RIPCI de la base de datos: [{id, sistema, periodicidad, texto, tipo}]
// Las operaciones NO están escritas en la plantilla: salen de la base de datos, de modo que un
// cambio normativo se aplica a todos los contratos nuevos sin tocar este fichero.

#let datos = json(bytes(sys.inputs.datos))
#let ops = json(bytes(sys.inputs.operaciones))

#let azul = rgb("#1f3a5f")
#let gris = luma(110)
#let linea = luma(200)
#let fondo = luma(245)

#let m = json(bytes(sys.inputs.empresa))
#let c = datos.cliente
#let e = datos.edificio
#let k = datos.contrato
#let excluidas = datos.at("excluir_operaciones", default: ())

#let periodos = (
  "3m": "Cada tres meses",
  "6m": "Cada seis meses",
  "1a": "Cada año",
  "3a": "Cada tres años",
  "5a": "Cada cinco años",
  "10a": "Cada diez años",
  "20a": "A los veinte años",
  "25a": "Cada veinticinco años",
  "vida_util": "Al término de la vida útil",
)
#let orden-preventivo = ("3m", "6m", "1a")

#let codigos(s) = s.at("subsistemas", default: (s.codigo,))
#let ops-de(s) = ops.filter(r => codigos(s).contains(r.sistema) and not excluidas.contains(r.id))

// ---------- Estilo general ----------
#set document(title: k.titulo, author: m.razon_social)
#set text(font: "Inter", size: 9.5pt, lang: "es", region: "es")
#set par(justify: true, leading: 0.65em, spacing: 0.9em)
#set heading(numbering: "1.")
#show heading.where(level: 1): it => block(above: 1.6em, below: 0.9em, sticky: true)[
  #set text(11pt, weight: "bold", fill: azul)
  #it
]
#show heading.where(level: 2): it => block(above: 1.2em, below: 0.6em, sticky: true)[
  #set text(9.5pt, weight: "bold")
  #it
]
#set table(stroke: (x, y) => (bottom: 0.5pt + linea), inset: (x: 6pt, y: 5pt))
#show table.cell.where(y: 0): set text(weight: "bold", size: 8.5pt, fill: azul)

#set page(
  paper: "a4",
  margin: (x: 2.2cm, top: 2.6cm, bottom: 2.4cm),
  header: context {
    if counter(page).get().first() > 1 {
      set text(8pt, fill: gris)
      [#m.razon_social #h(1fr) Contrato #k.referencia]
      v(-4pt)
      line(length: 100%, stroke: 0.5pt + linea)
    }
  },
  footer: context {
    set text(8pt, fill: gris)
    [#e.nombre #h(1fr) Página #counter(page).display() de #counter(page).final().first()]
  },
)

// ---------- Portada ----------
#page(header: none, footer: none)[
  #v(1fr)
  #text(9pt, fill: gris, tracking: 1pt)[#upper(m.razon_social)]
  #v(0.6em)
  #text(22pt, weight: "bold", fill: azul)[Contrato de mantenimiento]
  #v(0.2em)
  #text(13pt)[Protección contra incendios y ventilación]
  #v(2.2em)
  #line(length: 100%, stroke: 0.8pt + azul)
  #v(1.2em)
  #grid(
    columns: (3.2cm, 1fr),
    row-gutter: 0.9em,
    text(fill: gris)[Instalación], text(weight: "bold")[#e.nombre],
    text(fill: gris)[Situación], e.direccion,
    text(fill: gris)[Uso], e.uso,
    text(fill: gris)[Cliente], [#c.razon_social],
    text(fill: gris)[Referencia], k.referencia,
    text(fill: gris)[Fecha], k.lugar_fecha,
  )
  #v(2fr)
  #text(8pt, fill: gris)[
    #m.razon_social · CIF #m.cif · #m.domicilio \
    #for h in m.habilitaciones [#h.at(0): n.º #h.at(1) \ ]
  ]
]

// ---------- Cuerpo ----------
= Partes

De una parte, *#c.razon_social*, con CIF #c.cif y domicilio en #c.domicilio, en adelante *el titular*.

De otra parte, *#m.razon_social*, con CIF #m.cif y domicilio en #m.domicilio, en adelante *la empresa mantenedora*, habilitada como:

#table(
  columns: (1fr, auto),
  table.header[Habilitación][Número],
  ..m.habilitaciones.map(h => (h.at(0), h.at(1))).flatten(),
)

= Objeto

El presente contrato tiene por objeto el mantenimiento preventivo de las instalaciones de protección contra incendios y de ventilación del inmueble *#e.nombre*, situado en #e.direccion, en los términos que se detallan a continuación.

= Normativa aplicable

- Reglamento de instalaciones de protección contra incendios, aprobado por el Real Decreto 513/2017, de 22 de mayo, modificado por el Real Decreto 164/2025, de 4 de marzo (RIPCI), y en particular su anexo II.
- Normas UNE a las que remite el anexo II del RIPCI: UNE 23120 (extintores), UNE-EN 671-3 (BIE), UNE 23007-14 (detección y alarma).
- Reglamento de equipos a presión, aprobado por el Real Decreto 809/2021, de 21 de septiembre, en lo relativo a las pruebas de los extintores.
- Reglamento electrotécnico para baja tensión (REBT) y Código Técnico de la Edificación, DB HS 3, para la ventilación por monóxido de carbono.
- Instrucciones de mantenimiento de los fabricantes de los equipos.

= Alcance e inventario

Las instalaciones incluidas, con los equipos inventariados a la fecha del contrato, son las siguientes:

#for s in datos.sistemas [
  == #s.titulo
  #table(
    columns: (1fr, 2.2cm),
    align: (left, right),
    table.header[Equipo][Cantidad],
    ..s.equipos.map(q => (q.at(0), str(q.at(1)))).flatten(),
  )
]
#for o in datos.at("otros_sistemas", default: ()) [
  == #o.titulo
  #table(
    columns: (1fr, 2.2cm),
    align: (left, right),
    table.header[Equipo][Cantidad],
    ..o.equipos.map(q => (q.at(0), str(q.at(1)))).flatten(),
  )
]

Cualquier equipo que se añada o retire durante la vigencia del contrato se comunicará por escrito y podrá dar lugar a la revisión del importe.

= Programa de mantenimiento

Se realizarán, como mínimo, las operaciones del anexo II del RIPCI que se indican para cada sistema, además de las recomendadas por el fabricante. Las operaciones son las que establece el reglamento en su redacción vigente.

#for s in datos.sistemas {
  let lista = ops-de(s).filter(r => r.tipo == "preventivo")
  [== #s.titulo]
  if "norma_adicional" in s { par(text(style: "italic")[#s.norma_adicional]) }
  for p in orden-preventivo {
    let grupo = lista.filter(r => r.periodicidad == p)
    if grupo.len() > 0 {
      block(sticky: true, above: 0.9em, below: 0.4em, text(weight: "bold", fill: azul)[#periodos.at(p)])
      for r in grupo [- #r.texto]
    }
  }
}

#for o in datos.at("otros_sistemas", default: ()) [
  == #o.titulo
  #block(sticky: true, above: 0.9em, below: 0.4em, text(weight: "bold", fill: azul)[#o.periodicidad])
  #for t in o.operaciones [- #t]
]

= Visitas

Se realizarán *#datos.visitas.anuales visitas al año*. #datos.visitas.descripcion

Las fechas se acordarán con el titular. Las operaciones podrán adelantarse o retrasarse unos días respecto a la fecha prevista sin alterar el calendario, dejando constancia de la fecha real en el acta.

= Operaciones de periodicidad superior a un año

Las siguientes operaciones no están incluidas en el importe del contrato. La empresa mantenedora avisará al titular con antelación suficiente y presentará presupuesto cuando correspondan, según las fechas de fabricación, timbrado o puesta en servicio de cada equipo:

#{
  let filas = datos.sistemas
    .map(s => ops-de(s).filter(r => r.tipo != "preventivo"))
    .fold((), (a, b) => a + b)
  table(
    columns: (3.2cm, 1fr),
    table.header[Periodicidad][Operación],
    ..filas.map(r => (periodos.at(r.periodicidad), r.texto)).flatten(),
  )
}

= Documentación

Tras cada visita, la empresa mantenedora entregará al titular un certificado firmado por su responsable técnico, con las listas de comprobación de las operaciones realizadas, conforme al punto 5 del anexo II del RIPCI. Para su elaboración podrán utilizarse los formatos de la serie UNE 23580.

Tanto la empresa mantenedora como el titular conservarán constancia documental del cumplimiento del programa de mantenimiento durante al menos cinco años.

= Condiciones económicas

El importe anual del presente contrato asciende a *#datos.economia.importe_anual* (#datos.economia.importe_anual_letra), #datos.economia.iva.

Se facturarán *#datos.economia.importe_visita* en cada una de las #datos.visitas.anuales visitas anuales, #datos.condiciones.forma_pago.

= Condiciones generales

#set enum(numbering: "1.", spacing: 0.8em)
+ *Duración.* El contrato tendrá una duración de #datos.condiciones.duracion y se prorrogará tácitamente por periodos iguales. Cualquiera de las partes podrá resolverlo en cualquier momento comunicándolo por escrito con una antelación mínima de #datos.condiciones.preaviso, sin derecho a reclamación alguna.
+ *Revisión de precios.* #datos.condiciones.revision_precios
+ *Reparaciones.* Las reparaciones y sustituciones necesarias se realizarán previa aceptación por escrito del presupuesto correspondiente. Los materiales se facturarán según la tarifa vigente de la empresa mantenedora.
+ *Exclusiones.* Quedan fuera del importe: las operaciones de periodicidad superior a un año, las recargas de extintores por uso, las sustituciones por fin de vida útil, la reposición de componentes, las inspecciones periódicas por organismo de control que en su caso resulten exigibles y las actuaciones fuera del horario establecido.
+ *Horario.* Los trabajos se realizarán #m.horario.
+ *Averías.* La empresa mantenedora atenderá las averías comunicadas en un plazo máximo de #datos.condiciones.plazo_respuesta_averias. La reparación se facturará conforme a la condición 3.
+ *Subcontratación.* La empresa mantenedora podrá encargar parte de las operaciones a empresas mantenedoras habilitadas conforme al RIPCI, respondiendo ante el titular del cumplimiento del contrato.
+ *Obligaciones del titular.* Facilitar el acceso a las instalaciones, comunicar cualquier anomalía o modificación de los locales, no manipular los equipos y conservar la documentación del mantenimiento.
+ *Protección de datos.* Los datos personales facilitados se tratarán únicamente para la gestión de este contrato, conforme al Reglamento (UE) 2016/679 y a la Ley Orgánica 3/2018.

// ---------- Firmas ----------
#v(2em)
#block(breakable: false)[
  #text(weight: "bold")[En prueba de conformidad, ambas partes firman el presente contrato.]
  #v(1.2em)
  #let firma(titulo, empresa, f) = block(width: 100%)[
    #text(fill: gris, size: 8.5pt)[#upper(titulo)] \
    #text(weight: "bold")[#empresa]
    #v(3.2em)
    #line(length: 85%, stroke: 0.5pt + linea)
    #set text(8.5pt)
    #grid(columns: (1.6cm, 1fr), row-gutter: 0.7em,
      [Nombre:], f.nombre, [DNI:], f.dni, [Cargo:], f.cargo, [Fecha:], [])
  ]
  #grid(columns: (1fr, 1fr), column-gutter: 1.5cm,
    firma("El titular", c.razon_social, c.firmante),
    firma("La empresa mantenedora", m.razon_social, m.firmante),
  )
]
