// In a real app, the Rust core will either compile this template with the
// JSON injected, or write out a temporary .json file and compile.
#let data = json("view_model.json")

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

// A zig-zag/wavy line drawn inside a repeating pattern box
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

// ---------------------------------------------------------
// Bar Renderer
// ---------------------------------------------------------
#let render-bar(condition) = {
  // Convert 0-100 to a relative percentage for the box width
  let pct = condition.fill_percentage * 1%

  // Note: serde camelCase makes these lowercase first letter
  let fill-style = if condition.pattern == "solid" { solid-pattern } else if condition.pattern == "diagonalHatch" {
    hatch-pattern
  } else if condition.pattern == "crossedOut" { crossed-pattern } else if condition.pattern == "wavy" {
    wavy-pattern
  } else { solid-pattern } // Fallback

  // Outer box (the empty track), Inner box (the filled pattern)
  box(width: 100%, height: 1.2em, stroke: 0.5pt + luma(150), {
    box(width: pct, height: 100%, fill: fill-style, stroke: none)
  })
}

// ---------------------------------------------------------
// Document Layout & Table Generation
// ---------------------------------------------------------
#let conservation-report(data) = {
  set page("a4", margin: (x: 1.5cm, y: 2cm))
  set text(font: "Linux Libertine", size: 10pt)

  // Document Header
  align(center)[
    #text(size: 16pt, weight: "bold")[#data.project_name] \
    #v(0.5em)
    #text(size: 11pt, style: "italic")[Conservation Status Report]
  ]
  v(2em)

  // Initialize flat array for table cells
  let cells = (
    [*Item ID*],
    [*Condition Status*],
    [*Treatment Notes*],
    [*Material*],
    [*Location/Epoch*],
  )

  // Iterate over our View Model
  for row in data.rows {
    // Column 1 & 2 & 3: Always present
    cells.push(row.item_id)
    cells.push(render-bar(row.condition))
    cells.push(row.treatment_notes)

    // Column 4: Material Group (Merged Cell Logic)
    let mat = row.at("material_group", default: none)
    if mat != none {
      cells.push(table.cell(
        rowspan: mat.span,
        align: center + horizon,
      )[#mat.value])
    }

    // Column 5: Location Group (Merged Cell Logic)
    let loc = row.at("location_group", default: none)
    if loc != none {
      cells.push(table.cell(
        rowspan: loc.span,
        align: center + horizon,
      )[#loc.value])
    }
  }

  // Render the table
  table(
    columns: (15%, 25%, 30%, 15%, 15%),
    align: (left, left, left, center, center),
    stroke: (x, y) => if y == 0 { 1pt + black } else { 0.5pt + luma(200) },
    fill: (x, y) => if y == 0 { luma(240) } else { none },
    ..cells
  )
}

// Execute the report
#conservation-report(data)
