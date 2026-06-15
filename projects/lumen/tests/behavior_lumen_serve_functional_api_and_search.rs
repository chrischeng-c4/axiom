// SPEC-MANAGED: projects/lumen/external-contracts/behavior/serve-functional.md#lumen-serve-functional-api-and-search
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-serve-functional-api-and-search
// @capability search
// @claim query-planner-boolean-eval-roaring-postings
// @contract serve-functional-api-and-search-correctness
// @category behavior
// @required_for_production true
// @command cargo test -p lumen --test api_e2e --test vector_e2e --test planner_diff -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC placeholder: implement this external contract test or keep the manifest command authoritative"]
fn lumen_serve_functional_api_and_search() {
    panic!("AW EC placeholder for lumen-serve-functional-api-and-search");
}
// CODEGEN-END
