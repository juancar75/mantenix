// Tarifa propia de mantenimiento preventivo, calculada desde los costes del año.
#import "comun.typ": *

// Entradas: datos (JSON del motor) y empresa (JSON con razón social).
#let empresa = json(bytes(sys.inputs.empresa))

#let t = json(bytes(sys.inputs.datos))

#show: estilo.with(empresa: empresa.razon_social, "Tarifa de mantenimiento " + str(t.anio),
  subtitulo: "Precio por unidad y revisión · coste hora oficial " + eur(t.coste_hora_oficial)
    + " · recargo " + num(t.recargo_pct, d: 0) + " % · margen " + num(t.margen_pct, d: 0) + " %")

#for fam in ("RITE", "PCI") [
  = #(if fam == "RITE" { "Instalaciones térmicas (RITE)" } else { "Protección contra incendios (RIPCI)" })
  #table(columns: (auto, 1fr, auto, auto, auto, auto, auto), align: (left, left, left, right, right, right, right),
    table.header[Código][Equipo][Origen][Min/rev.][Coste/rev.][*Precio/rev.*][Precio/año#super[1]],
    ..t.filas.filter(f => f.familia == fam).map(f => (
      text(7.5pt, f.codigo), f.descripcion, text(7.5pt, f.origen),
      if f.minutos_revision > 0 { num(f.minutos_revision, d: 0) } else { "—" },
      eur(f.coste_revision), strong(eur(f.precio_revision)), eur(f.precio_anual_regimen),
    )).flatten(),
  )
]

#text(7.5pt, fill: gris)[
  #super[1] Precio anual con las periodicidades reglamentarias completas (tabla 3.3 del RITE para >70 kW, 12 meses de uso; 4 revisiones en PCI).
  Precios sin IVA ni desplazamiento. El desplazamiento y el tiempo fijo por visita se añaden en la valoración de cada contrato.
]
