// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-src.md#schema
// CODEGEN-BEGIN
fn main() -> anyhow::Result<()> {
    let matches = jet::cli::command()
        .name("jet")
        .version(env!("CARGO_PKG_VERSION"))
        .get_matches();

    jet::cli::execute(&matches)
}
// CODEGEN-END
