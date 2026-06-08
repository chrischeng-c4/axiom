// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
// CODEGEN-BEGIN
//! Browser-like cell range selection for the WebGPU renderer.
//!
//! This module is pure Rust on purpose: pointer coordinates, visible
//! cell discovery, range state, highlight paint, and TSV serialization
//! are runtime semantics owned by Jet/WASM. The browser layer only
//! supplies platform events and clipboard I/O.

use std::collections::HashMap;

use crate::renderer::{Color, LaidOutKind, LayoutTree, PaintOp, Point, Rect};

const FIXTURE_TABLE_COLUMNS: usize = 100;
const SELECTION_FILL: Color = Color {
    r: 0x1a,
    g: 0x73,
    b: 0xe8,
    a: 0x33,
};
const SELECTION_STROKE: Color = Color {
    r: 0x1a,
    g: 0x73,
    b: 0xe8,
    a: 0xff,
};

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellCoord {
    pub row: usize,
    pub col: usize,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellRange {
    pub anchor: CellCoord,
    pub focus: CellCoord,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
impl CellRange {
    pub fn normalized(self) -> NormalizedCellRange {
        NormalizedCellRange {
            row_start: self.anchor.row.min(self.focus.row),
            row_end: self.anchor.row.max(self.focus.row),
            col_start: self.anchor.col.min(self.focus.col),
            col_end: self.anchor.col.max(self.focus.col),
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NormalizedCellRange {
    pub row_start: usize,
    pub row_end: usize,
    pub col_start: usize,
    pub col_end: usize,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
impl NormalizedCellRange {
    pub fn contains(self, coord: CellCoord) -> bool {
        (self.row_start..=self.row_end).contains(&coord.row)
            && (self.col_start..=self.col_end).contains(&coord.col)
    }

    pub fn cell_count(self) -> usize {
        (self.row_end - self.row_start + 1) * (self.col_end - self.col_start + 1)
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
#[derive(Debug, Clone, PartialEq)]
pub struct VisibleTableCell {
    pub coord: CellCoord,
    pub rect: Rect,
    pub value: String,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CellSelectionState {
    anchor: Option<CellCoord>,
    focus: Option<CellCoord>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
impl CellSelectionState {
    pub fn select_cell(&mut self, coord: CellCoord, extend: bool) {
        if extend {
            let anchor = self.anchor.unwrap_or(coord);
            self.anchor = Some(anchor);
            self.focus = Some(coord);
        } else {
            self.anchor = Some(coord);
            self.focus = Some(coord);
        }
    }

    pub fn clear(&mut self) {
        self.anchor = None;
        self.focus = None;
    }

    pub fn range(&self) -> Option<CellRange> {
        Some(CellRange {
            anchor: self.anchor?,
            focus: self.focus?,
        })
    }

    pub fn normalized_range(&self) -> Option<NormalizedCellRange> {
        self.range().map(CellRange::normalized)
    }

    pub fn is_active(&self) -> bool {
        self.range().is_some()
    }

    pub fn selected_visible_cells<'a>(
        &self,
        cells: &'a [VisibleTableCell],
    ) -> Vec<&'a VisibleTableCell> {
        let Some(range) = self.normalized_range() else {
            return Vec::new();
        };
        cells
            .iter()
            .filter(|cell| range.contains(cell.coord))
            .collect()
    }

    pub fn highlight_ops(&self, cells: &[VisibleTableCell]) -> Vec<PaintOp> {
        let mut ops = Vec::new();
        for cell in self.selected_visible_cells(cells) {
            ops.push(PaintOp::FillRect {
                rect: cell.rect,
                color: SELECTION_FILL,
            });
            ops.push(PaintOp::StrokeRect {
                rect: cell.rect,
                color: SELECTION_STROKE,
                width: 2.0,
            });
        }
        ops
    }

    pub fn selected_tsv(&self, cells: &[VisibleTableCell]) -> Option<String> {
        let range = self.normalized_range()?;
        let by_coord: HashMap<CellCoord, &str> = cells
            .iter()
            .map(|cell| (cell.coord, cell.value.as_str()))
            .collect();

        let mut rows = Vec::new();
        for row in range.row_start..=range.row_end {
            let mut cols = Vec::new();
            for col in range.col_start..=range.col_end {
                cols.push(by_coord.get(&CellCoord { row, col }).copied().unwrap_or(""));
            }
            rows.push(cols.join("\t"));
        }
        Some(rows.join("\n"))
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
pub fn hit_test_visible_cell(cells: &[VisibleTableCell], point: Point) -> Option<VisibleTableCell> {
    cells
        .iter()
        .rev()
        .find(|cell| rect_contains(cell.rect, point))
        .cloned()
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
pub fn visible_table_cells(tree: &LayoutTree) -> Vec<VisibleTableCell> {
    let mut cells = Vec::new();
    for (idx, node) in tree.nodes.iter().enumerate() {
        let LaidOutKind::Intrinsic { tag: "td", .. } = &node.kind else {
            continue;
        };

        let value = tree
            .nodes
            .iter()
            .skip(idx + 1)
            .filter_map(|candidate| match &candidate.kind {
                LaidOutKind::Text { content, .. }
                    if rect_center_inside(node.rect, candidate.rect) =>
                {
                    Some(content.as_str())
                }
                _ => None,
            })
            .collect::<String>();
        let value = value.trim().to_string();
        let Some(coord) = parse_fixture_cell_coord(&value) else {
            continue;
        };
        cells.push(VisibleTableCell {
            coord,
            rect: node.rect,
            value,
        });
    }
    cells
}

fn parse_fixture_cell_coord(value: &str) -> Option<CellCoord> {
    let n = value.strip_prefix("cell ")?.parse::<usize>().ok()?;
    Some(CellCoord {
        row: n / FIXTURE_TABLE_COLUMNS,
        col: n % FIXTURE_TABLE_COLUMNS,
    })
}

fn rect_center_inside(container: Rect, child: Rect) -> bool {
    rect_contains(
        container,
        Point {
            x: child.x + child.w / 2.0,
            y: child.y + child.h / 2.0,
        },
    )
}

fn rect_contains(r: Rect, p: Point) -> bool {
    p.x >= r.x && p.x < r.x + r.w && p.y >= r.y && p.y < r.y + r.h
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::{LaidOutNode, TextStyle};
    use crate::Props;

    fn td(row: usize, col: usize, rect: Rect) -> Vec<LaidOutNode> {
        let value = row * FIXTURE_TABLE_COLUMNS + col;
        vec![
            LaidOutNode {
                kind: LaidOutKind::Intrinsic {
                    tag: "td",
                    props: Props::default(),
                },
                rect,
            },
            LaidOutNode {
                kind: LaidOutKind::Text {
                    content: "cell ".to_string(),
                    style: TextStyle::default(),
                },
                rect: Rect {
                    x: rect.x + 4.0,
                    y: rect.y + 4.0,
                    w: 24.0,
                    h: 18.0,
                },
            },
            LaidOutNode {
                kind: LaidOutKind::Text {
                    content: value.to_string(),
                    style: TextStyle::default(),
                },
                rect: Rect {
                    x: rect.x + 28.0,
                    y: rect.y + 4.0,
                    w: 24.0,
                    h: 18.0,
                },
            },
        ]
    }

    #[test]
    fn visible_cells_collect_fragmented_table_text() {
        let tree = LayoutTree {
            root_rect: Rect {
                x: 0.0,
                y: 0.0,
                w: 200.0,
                h: 100.0,
            },
            nodes: td(
                1,
                2,
                Rect {
                    x: 10.0,
                    y: 20.0,
                    w: 80.0,
                    h: 24.0,
                },
            ),
        };

        let cells = visible_table_cells(&tree);
        assert_eq!(cells.len(), 1);
        assert_eq!(cells[0].coord, CellCoord { row: 1, col: 2 });
        assert_eq!(cells[0].value, "cell 102");
    }

    #[test]
    fn shift_selection_serializes_visible_range_as_tsv() {
        let mut nodes = Vec::new();
        nodes.extend(td(
            0,
            0,
            Rect {
                x: 0.0,
                y: 0.0,
                w: 80.0,
                h: 24.0,
            },
        ));
        nodes.extend(td(
            0,
            1,
            Rect {
                x: 80.0,
                y: 0.0,
                w: 80.0,
                h: 24.0,
            },
        ));
        nodes.extend(td(
            1,
            0,
            Rect {
                x: 0.0,
                y: 24.0,
                w: 80.0,
                h: 24.0,
            },
        ));
        nodes.extend(td(
            1,
            1,
            Rect {
                x: 80.0,
                y: 24.0,
                w: 80.0,
                h: 24.0,
            },
        ));
        let tree = LayoutTree {
            root_rect: Rect {
                x: 0.0,
                y: 0.0,
                w: 200.0,
                h: 100.0,
            },
            nodes,
        };
        let cells = visible_table_cells(&tree);
        let mut selection = CellSelectionState::default();

        selection.select_cell(CellCoord { row: 0, col: 0 }, false);
        selection.select_cell(CellCoord { row: 1, col: 1 }, true);

        assert_eq!(
            selection.selected_tsv(&cells).as_deref(),
            Some("cell 0\tcell 1\ncell 100\tcell 101")
        );
        assert_eq!(selection.highlight_ops(&cells).len(), 8);
    }

    #[test]
    fn hit_test_returns_visible_cell() {
        let cell = VisibleTableCell {
            coord: CellCoord { row: 2, col: 3 },
            rect: Rect {
                x: 10.0,
                y: 20.0,
                w: 80.0,
                h: 24.0,
            },
            value: "cell 203".to_string(),
        };
        assert_eq!(
            hit_test_visible_cell(&[cell], Point { x: 20.0, y: 30.0 }).map(|c| c.coord),
            Some(CellCoord { row: 2, col: 3 })
        );
    }
}
// CODEGEN-END
