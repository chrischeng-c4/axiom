// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
// CODEGEN-BEGIN
//! Canvas 2D backend — executes `PaintOp`s against a
//! `web_sys::CanvasRenderingContext2d`. Feature-gated behind `canvas`
//! so the crate's default build compiles in plain `cargo test`.

use super::{Color, PaintBackend, PaintOp};

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
pub struct CanvasBackend {
    ctx: web_sys::CanvasRenderingContext2d,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
impl CanvasBackend {
    pub fn new(ctx: web_sys::CanvasRenderingContext2d) -> Self {
        Self { ctx }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
impl PaintBackend for CanvasBackend {
    fn execute(&mut self, ops: &[PaintOp]) {
        for op in ops {
            match op {
                PaintOp::FillRect { rect, color } => {
                    self.ctx.set_fill_style_str(&css_color(*color));
                    self.ctx
                        .fill_rect(rect.x as f64, rect.y as f64, rect.w as f64, rect.h as f64);
                }
                PaintOp::StrokeRect { rect, color, width } => {
                    self.ctx.set_stroke_style_str(&css_color(*color));
                    self.ctx.set_line_width(*width as f64);
                    self.ctx.stroke_rect(
                        rect.x as f64,
                        rect.y as f64,
                        rect.w as f64,
                        rect.h as f64,
                    );
                }
                PaintOp::Text {
                    origin,
                    content,
                    font,
                    color,
                } => {
                    self.ctx.set_fill_style_str(&css_color(*color));
                    self.ctx.set_font(&format!(
                        "{weight} {size}px {family}",
                        weight = font.weight,
                        size = font.size_px,
                        family = font.family
                    ));
                    self.ctx.set_text_baseline("alphabetic");
                    let _ = self
                        .ctx
                        .fill_text(content, origin.x as f64, origin.y as f64);
                }
                PaintOp::PushClip { rect } => {
                    self.ctx.save();
                    self.ctx.begin_path();
                    self.ctx
                        .rect(rect.x as f64, rect.y as f64, rect.w as f64, rect.h as f64);
                    self.ctx.clip();
                }
                PaintOp::PopClip => {
                    self.ctx.restore();
                }
            }
        }
    }
}

fn css_color(c: Color) -> String {
    if c.a == 255 {
        format!("rgb({}, {}, {})", c.r, c.g, c.b)
    } else {
        format!("rgba({}, {}, {}, {:.4})", c.r, c.g, c.b, c.a as f32 / 255.0)
    }
}
// CODEGEN-END
