#let data = json.decode(sys.inputs.json_data)

// ---------------------------------------------------------
// Define Patterns for the Condition Bars
// ---------------------------------------------------------
#let hatch-pattern = pattern(size: (8pt, 8pt))[
  #line(start: (0pt, 8pt), end: (8pt, 0pt), stroke: 0.5pt + black)
]
#let crossed-pattern = pattern(size: (8pt, 8pt))[
  #line(start: (0pt, 0pt), end: (8pt, 8pt), stroke: 0.5pt + black)
  #line(start: (0pt, 8pt), end: (8pt, 0pt), stroke: 0.5pt + black)
]
#let wavy-pattern = pattern(size: (12pt, 12pt))[
  #path(
    stroke: 0.5pt + black,
    (0pt, 6pt),
    (3pt, 2pt),
    (9pt, 10pt),
    (12pt, 6pt),
  )
]
#let solid-pattern = rgb("555555")

// Function to rotate column header
#let rotated-header(content) = {
  align(center + bottom, rotate(-90deg, reflow: true)[
    #box(width: 5.5em, align(center)[*#content*])
  ])
}
// ---------------------------------------------------------
// Bar Renderer
// ---------------------------------------------------------
#let render-bar(condition) = {
  let fill-style = if condition.pattern == "solid" { solid-pattern } else if condition.pattern == "diagonalHatch" {
    hatch-pattern
  } else if condition.pattern == "crossedOut" { crossed-pattern } else if condition.pattern == "wavy" {
    wavy-pattern
  } else { solid-pattern }

  box(width: 100%, height: 1.2em, stroke: 0.5pt + luma(150), {
    for slot in condition.slots {
      let x0 = slot.start_pct * 1%
      let w = calc.max(slot.end_pct - slot.start_pct, 0.3) * 1%
      place(left, dx: x0, box(width: w, height: 100%, fill: fill-style, stroke: none))
    }
  })
}

// ---------------------------------------------------------
// Document Layout & Table Generation
// ---------------------------------------------------------
#let conservation-report(data) = {
  set page("a4", margin: (x: 1.5cm, y: 2cm))
  set text(font: "Linux Libertine", size: 10pt)
  align(center)[
    #text(size: 16pt, weight: "bold")[#data.project_name] \
    #v(0.5em)
    #text(size: 11pt, style: "italic")[Conservation Status Report]
  ]
  v(2em)

  let base-ratios = (5, 35, 10, 20, 10, 10, 10)
  let base-aligns = (center, center, center, left, center, center, center)

  let active-ratios = ()
  let active-aligns = ()
  for (i, is-visible) in data.visible_columns.enumerate() {
    if is-visible {
      active-ratios.push(base-ratios.at(i))
      active-aligns.push(base-aligns.at(i))
    }
  }

  let total-ratio = active-ratios.sum()
  let active-columns = active-ratios.map(r => (r / total-ratio) * 100%)

  let all-headers = (
    rotated-header(data.translations.col_no),
    [*#data.translations.col_graphical*],
    [*#data.translations.col_woven*],
    [*#data.translations.col_char*],
    rotated-header(data.translations.col_thickness),
    rotated-header(data.translations.col_chrono),
    rotated-header(data.translations.col_dating),
  )

  let cells = ()
  for (i, is-visible) in data.visible_columns.enumerate() {
    if is-visible { cells.push(all-headers.at(i)) }
  }

  for row in data.rows {
    if data.visible_columns.at(0) { cells.push(str(row.row_number)) }
    if data.visible_columns.at(1) { cells.push(render-bar(row.graphical_representation)) }
    if data.visible_columns.at(2) { cells.push(render-bar(row.woven_trim)) }
    if data.visible_columns.at(3) { cells.push(row.layer_characteristics) }
    if data.visible_columns.at(4) { cells.push(row.layer_thickness) }

    if data.visible_columns.at(5) {
      let chrono = row.at("chronological_phase", default: none)
      if chrono != none {
        cells.push(table.cell(rowspan: chrono.span, align: center + horizon)[#chrono.value])
      }
    }

    if data.visible_columns.at(6) {
      let dating = row.at("dating_period", default: none)
      if dating != none {
        cells.push(table.cell(rowspan: dating.span, align: center + horizon)[#dating.value])
      }
    }
  }

  table(
    columns: active-columns,
    align: active-aligns,
    stroke: (x, y) => if y == 0 { 1pt + black } else { 0.5pt + luma(200) },
    fill: (x, y) => if y == 0 { luma(240) } else { none },
    ..cells
  )
}

#conservation-report(data)
