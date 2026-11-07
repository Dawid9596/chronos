#let data = json.decode(sys.inputs.json_data)
#let params = data.canvas_params

// ---------------------------------------------------------
// Define Patterns for the Condition Bars
// ---------------------------------------------------------
#let vertical-hatch = pattern(size: (4pt, 10pt))[
  #line(start: (0pt, 0pt), end: (0pt, 10pt), stroke: 0.5pt + black)
]
#let dense-grid = pattern(size: (5pt, 5pt))[
  #line(start: (0pt, 5pt), end: (5pt, 0pt), stroke: 0.5pt + black)
  #line(start: (0pt, 0pt), end: (5pt, 5pt), stroke: 0.5pt + black)
]
// Bolus: rings pattern, embedded from patterns/bolus.svg (viewBox 3108x343).
// Tile height 12pt keeps the aspect ratio of the source SVG: 12pt * 3108/343 ~= 108.5pt.
#let bolus-hatch = pattern(size: (108.5pt, 12pt))[
  #image("bolus.svg", width: 108.5pt, height: 12pt)
]
// Brick: embedded from patterns/cegla.svg (viewBox 3108x299); 12pt height keeps the aspect.
#let brick = pattern(size: (12pt * 3108 / 299, 12pt))[
  #image("cegla.svg", width: 12pt * 3108 / 299, height: 12pt)
]
// Ceramics: embedded from patterns/ceramika.svg (viewBox 2845x321); 12pt height keeps the aspect.
#let ceramics = pattern(size: (12pt * 2845 / 321, 12pt))[
  #image("ceramika.svg", width: 12pt * 2845 / 321, height: 12pt)
]
// Wood: embedded from patterns/drewno.svg (viewBox 9436x1039); 12pt height keeps the aspect.
#let wood = pattern(size: (12pt * 9436 / 1039, 12pt))[
  #image("drewno.svg", width: 12pt * 9436 / 1039, height: 12pt)
]
// #let hatch-pattern = pattern(size: (8pt, 8pt))[
//   #line(start: (0pt, 8pt), end: (8pt, 0pt), stroke: 0.5pt + black)
// ]
// #let crossed-pattern = pattern(size: (8pt, 8pt))[
//   #line(start: (0pt, 0pt), end: (8pt, 8pt), stroke: 0.5pt + black)
//   #line(start: (0pt, 8pt), end: (8pt, 0pt), stroke: 0.5pt + black)
// ]
// #let wavy-pattern = pattern(size: (12pt, 12pt))[
//   #path(
//     stroke: 0.5pt + black,
//     (0pt, 6pt),
//     (3pt, 2pt),
//     (9pt, 10pt),
//     (12pt, 6pt),
//   )
// ]
// #let solid-pattern = rgb("555555")

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
  let fill-style = if condition.pattern == "verticalHatch" { 
    vertical-hatch 
  } else if condition.pattern == "denseGrid" {
    dense-grid
  } else if condition.pattern == "bolus" {
    bolus-hatch
  } else if condition.pattern == "brick" {
    brick
  } else if condition.pattern == "ceramics" {
    ceramics
  } else if condition.pattern == "wood" {
    wood
  } else { 
    none 
  }

  box(width: 100%, height: 1.2em, stroke: 0.5pt + luma(150), {
    for slot in condition.slots {
      let span = slot.end_pct - slot.start_pct
      let x0 = slot.start_pct * 1%
      if calc.abs(span) < 0.5 {
        place(
          left,
          dx: x0,
          line(
            start: (0pt, 0%),
            end: (0pt, 100%),
            stroke: (params.zero_width_line_pt * 1pt) + luma(77),
          ),
        )
      } else {
        let w = span * 1%
        place(left, dx: x0, box(width: w, height: 100%, fill: fill-style, stroke: none))
      }
    }
  })
}

// ---------------------------------------------------------
// Document Layout & Table Generation
// ---------------------------------------------------------
#let conservation-report(data) = {
  let page-margin-val = params.page_margin_mm * 1mm
  set page(
    lower(params.paper_size),
    margin: (x: page-margin-val, y: page-margin-val),
    flipped: params.landscape
  )

  // Cell padding
  let cell-inset-val = params.cell_padding_mm * 1mm
  set text(font: "Linux Libertine", size: 10pt)
  align(center)[
    #text(size: 16pt, weight: "bold")[#data.project_name] \
    #v(0.5em)
    #text(size: 11pt, style: "italic")[Conservation Status Report]
  ]
  v(2em)

  let base-ratios = params.column_ratios
  let base-aligns = (center, center, center, left, center, center, center)

  let active-ratios = ()
  let active-aligns = ()
  for (i, is-visible) in data.visible_columns.enumerate() {
    if is-visible {
      active-ratios.push(base-ratios.at(i))
      active-aligns.push(base-aligns.at(i))
    }
  }

  let total-ratio = if active-ratios.len() == 0 { 1 } else { active-ratios.sum() }
  let active-columns = if active-ratios.len() == 0 { () } else { active-ratios.map(r => (r / total-ratio) * 100%) }

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

  if active-ratios.len() > 0 {
    table(
      inset: cell-inset-val,
      columns: active-columns,
      align: active-aligns,
      stroke: (x, y) => if y == 0 { 1pt + black } else { 0.5pt + luma(200) },
      fill: (x, y) => if y == 0 { luma(240) } else { none },
      ..cells
    )
  }
}

#conservation-report(data)
