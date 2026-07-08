// SPEC-MANAGED: projects/tape/tech-design/semantic/source/projects-tape-build-rs.md#logic
// <HANDWRITE gap="missing-generator:project-bootstrap" tracker="#768" reason="Build-stamp wiring for the initial Tape CLI.">
fn main() {
    build_stamp::stamp("TAPE");
}
// </HANDWRITE>
