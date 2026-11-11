#let data = json.decode(sys.inputs.json_data)
#let params = data.canvas_params

// ---------------------------------------------------------
// Define Patterns for the Condition Bars
// ---------------------------------------------------------
#let svg-tiles = (
  bolus: ("bolus.svg", 3108 / 343),
  brick: ("cegla.svg", 3108 / 299),
  ceramics: ("ceramika.svg", 2845 / 321),
  wood: ("drewno.svg", 9436 / 1039),
  textile: ("Tkanina, płótno, pasy tapicerskie.svg", 2371 / 959),
  metalFoil: ("folie metalowe, złocenia, srebrzenia.svg", 3107 / 351),
  embroidery: ("haft.svg", 3108 / 343),
  stone: ("kamień.svg", 3081 / 329),
  glue: ("klej.svg", 3032 / 343),
  bone: ("kość kamienie szlachetne.svg", 2961 / 329),
  sprayVarnish: ("laserunek.svg", 609 / 341),
  painting: ("malowidło, przemalowania, retusz.svg", 1349 / 331),
  metal: ("metal, blacha, gwoździe.svg", 751 / 343),
  paper: ("papier.svg", 1697 / 309),
  patina: ("patyna.svg", 3108 / 330),
  parchment: ("pergamin.svg", 2985 / 343),
  whitewash: ("pobiala.svg", 3108 / 308),
  leather: ("skora.svg", 2833 / 333),
  stucco: ("stiuk.svg", 3028 / 343),
  glaze: ("szkliwo.svg", 2864 / 343),
  plasterwork: ("sztukateria, odlewy i aplikacje gipsowe.svg", 9692 / 1070),
  cardboard: ("tektura.svg", 1752 / 343),
  plastic: ("tworzywo sztuczne.svg", 2934 / 330),
  mixedThread: ("watek mieszany.svg", 2801 / 338),
  varnish: ("werniks.svg", 2918 / 347),
  wax: ("wosk, masy dublazowe.svg", 2721 / 340),
  nonwoven: ("włóknina.svg", 2759 / 343),
  mortar: ("zaprawa tynk.svg", 1711 / 343),
  goldPowder: ("zloto w proszku.svg", 3082 / 340),
)

#let bar-fill(name, height) = {
  if name in svg-tiles {
    let (file, aspect) = svg-tiles.at(name)
    let w = height * aspect
    pattern(size: (w, height), image(file, width: w, height: height))
  }
}

// Function to rotate column header
#let rotated-header(content) = {
  align(center + bottom, rotate(-90deg, reflow: true)[
    #box(width: 5.5em, align(center)[*#content*])
  ])
}
// ---------------------------------------------------------
// Bar Renderer
// ---------------------------------------------------------
// Height a bar contributes to its row when nothing else makes the row taller.
#let bar-min-height = 1.2em
#let bar-stroke = 0.5pt

#let render-bar(condition) = layout(size => {
  box(width: size.width, height: size.height, inset: bar-stroke / 2, {
    let bar-h = size.height - bar-stroke
    box(width: 100%, height: 100%, stroke: bar-stroke + luma(150), {
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
          place(left, dx: x0, box(
            width: w,
            height: 100%,
            fill: bar-fill(condition.pattern, bar-h),
            stroke: none,
          ))
        }
      }
    })
  })
})

// ---------------------------------------------------------
// Document Layout & Table Generation
// ---------------------------------------------------------
#let conservation-report(data) = {
  let page-margin-val = params.page_margin_mm * 1mm
  set page(
    lower(params.paper_size),
    margin: (x: page-margin-val, y: page-margin-val),
    flipped: params.landscape,
  )

  // Cell padding
  let cell-inset-val = params.cell_padding_mm * 1mm
  let bar-inset-val = params.at("bar_padding_mm", default: params.cell_padding_mm) * 1mm
  set text(font: "Linux Libertine", size: 10pt)
  align(center)[
    #text(size: 16pt, weight: "bold")[#data.project_name]
    #if data.subtitle != "" [
      \
      #v(0.5em)
      #text(size: 11pt, style: "italic")[#data.subtitle]
    ]
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
    if data.visible_columns.at(1) {
      cells.push(table.cell(inset: bar-inset-val)[#render-bar(row.graphical_representation)])
    }
    if data.visible_columns.at(2) { cells.push(table.cell(inset: bar-inset-val)[#render-bar(row.woven_trim)]) }
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
    layout(avail => {
      let col-w = active-ratios.map(r => r / total-ratio * avail.width)
      let vis = data.visible_columns

      let active-index(col) = vis.slice(0, col).filter(v => v).len()

      // Row heights: Size each row
      // here, from a one-row copy of the grid (probe) in which the bar cells are a
      // placeholder of the minimum bar height, to then give the table explicit rows.
      let row-height(row) = {
        let probe = ()
        if vis.at(0) { probe.push(str(row.row_number)) }
        if vis.at(1) { probe.push(grid.cell(inset: bar-inset-val, box(height: bar-min-height))) }
        if vis.at(2) { probe.push(grid.cell(inset: bar-inset-val, box(height: bar-min-height))) }
        if vis.at(3) { probe.push(row.layer_characteristics) }
        if vis.at(4) { probe.push(row.layer_thickness) }
        if vis.at(5) { probe.push([]) }
        if vis.at(6) { probe.push([]) }
        measure(grid(
          columns: col-w,
          inset: cell-inset-val,
          align: active-aligns,
          ..probe
        )).height
      }
      let heights = data.rows.map(row-height)

      // Merged cells: combined height logic
      for (i, row) in data.rows.enumerate() {
        for (col, key) in ((5, "chronological_phase"), (6, "dating_period")) {
          let group = if vis.at(col) { row.at(key, default: none) } else { none }
          if group != none {
            let inner = col-w.at(active-index(col)) - 2 * cell-inset-val
            let need = measure(block(width: inner, group.value)).height + 2 * cell-inset-val
            let have = heights.slice(i, i + group.span).sum()
            if need > have {
              for j in range(i, i + group.span) {
                heights.at(j) += (need - have) / group.span
              }
            }
          }
        }
      }

      table(
        inset: cell-inset-val,
        columns: col-w,
        rows: (auto, ..heights),
        align: active-aligns,
        stroke: (x, y) => if y == 0 { 1pt + black } else { 0.5pt + luma(200) },
        fill: (x, y) => if y == 0 { luma(240) } else { none },
        ..cells
      )
    })
  }

  // Artwork attribution — required by CC BY-SA 4.0 (patterns/LICENSE).
  v(1.2em)
  align(right, text(
    size: 7pt,
    fill: luma(110),
    style: "italic",
    [#data.translations.artwork_attribution],
  ))
}

#conservation-report(data)
