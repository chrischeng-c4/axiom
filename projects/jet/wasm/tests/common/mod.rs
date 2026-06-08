// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-tests.md#tests
// CODEGEN-BEGIN
use jet_wasm::text::FontFace;

pub const TUFFY_REGULAR: &[u8] = include_bytes!("../fixtures/fonts/tuffy/Tuffy.ttf");

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-tests.md#tests
pub fn tuffy_regular() -> FontFace {
    FontFace::from_bytes(TUFFY_REGULAR).expect("vendored Tuffy Regular must load")
}
// CODEGEN-END
