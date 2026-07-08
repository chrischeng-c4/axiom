// SPEC-MANAGED: apps/tape/tech-design/semantic/source/apps-tape-build-rs.md#logic
// <HANDWRITE gap="missing-generator:project-bootstrap" tracker="#768" reason="Build-stamp wiring for the initial Tape CLI.">
fn main() {
    build_stamp::stamp("TAPE");
}
// </HANDWRITE>
