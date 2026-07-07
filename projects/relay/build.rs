// HANDWRITE-BEGIN gap="missing-generator:logic:eeba4423" tracker="pending-tracker" reason="Delegate to build_stamp::stamp('RELAY') so RELAY_GIT_SHA/RELAY_BUILT_AT/RELAY_TARGET feed ToolInfo — no hand-rolled git/timestamp logic."
//! Build script: stamp `RELAY_GIT_SHA`, `RELAY_BUILT_AT`, and `RELAY_TARGET`
//! into the binary so the standard CLI ops (`upgrade` picks the matching
//! release asset; `issue` reports provenance) work without a server. The
//! stamping logic lives in the shared `libs/build-stamp` crate; this file
//! only supplies relay's `RELAY` env-var prefix.

fn main() {
    build_stamp::stamp("RELAY");
}

<!-- marker: missing-generator:logic:eeba4423 path: projects/relay/build.rs reason: Delegate to build_stamp::stamp('RELAY') so RELAY_GIT_SHA/RELAY_BUILT_AT/RELAY_TARGET feed ToolInfo — no hand-rolled git/timestamp logic. -->
// HANDWRITE-END
