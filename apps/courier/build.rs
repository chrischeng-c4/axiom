// HANDWRITE-BEGIN gap="missing-generator:logic:c0ur1e00" tracker="pending-tracker" reason="Delegate to build_stamp::stamp('COURIER') so COURIER_GIT_SHA/COURIER_BUILT_AT/COURIER_TARGET feed ToolInfo — no hand-rolled git/timestamp logic."
//! Build script: stamp `COURIER_GIT_SHA`, `COURIER_BUILT_AT`, and
//! `COURIER_TARGET` into the binary so the standard CLI ops (`upgrade` picks
//! the matching release asset; `issue` reports provenance) work without a
//! server. The stamping logic lives in the shared `libs/build-stamp` crate;
//! this file only supplies courier's `COURIER` env-var prefix.

fn main() {
    build_stamp::stamp("COURIER");
}
// HANDWRITE-END
