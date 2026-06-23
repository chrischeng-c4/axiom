//! Golden-snapshot and type-check gates for `jet codegen openapi`.
//!
//! @spec .aw/tech-design/projects/jet/interfaces/cli/openapi-client-codegen-types-fetch-client-react-query-hooks.md#unit-test

use jet::codegen::{generate, GenOptions, GeneratedOutput, HttpClient};
use std::path::PathBuf;
use std::process::Command;

fn full_opts() -> GenOptions {
    GenOptions {
        spec_path: PathBuf::new(),
        out_dir: PathBuf::new(),
        client_name: "createClient".to_string(),
        http_client: HttpClient::Fetch,
        emit_types: true,
        emit_client: true,
        emit_hooks: true,
    }
}

fn file<'a>(out: &'a GeneratedOutput, name: &str) -> &'a str {
    out.files
        .iter()
        .find(|f| f.rel_path == name)
        .map(|f| f.contents.as_str())
        .unwrap_or_else(|| panic!("missing generated file {name}"))
}

/// Byte-for-byte golden comparison for the minimal fixture. Regenerate with:
/// `jet codegen openapi projects/jet/tests/fixtures/codegen/minimal.json
///  --out projects/jet/tests/__snapshots__/codegen` (then rename `X.ts` to
/// `minimal__X.ts`).
#[test]
fn minimal_matches_golden_snapshots() {
    let spec = include_str!("../fixtures/codegen/minimal.json");
    let out = generate(spec, &full_opts()).expect("generate minimal");

    assert_eq!(
        file(&out, "types.ts"),
        include_str!("../__snapshots__/codegen/minimal__types.ts")
    );
    assert_eq!(
        file(&out, "runtime.ts"),
        include_str!("../__snapshots__/codegen/minimal__runtime.ts")
    );
    assert_eq!(
        file(&out, "client.ts"),
        include_str!("../__snapshots__/codegen/minimal__client.ts")
    );
    assert_eq!(
        file(&out, "hooks.ts"),
        include_str!("../__snapshots__/codegen/minimal__hooks.ts")
    );
    assert_eq!(
        file(&out, "index.ts"),
        include_str!("../__snapshots__/codegen/minimal__index.ts")
    );
}

/// `--http axios` swaps only `runtime.ts`; every other file is byte-identical to
/// the fetch goldens. The axios runtime is golden-checked separately.
#[test]
fn axios_backend_matches_golden_and_is_surface_invariant() {
    let spec = include_str!("../fixtures/codegen/minimal.json");
    let mut opts = full_opts();
    opts.http_client = HttpClient::Axios;
    let out = generate(spec, &opts).expect("generate minimal (axios)");

    assert_eq!(
        file(&out, "runtime.ts"),
        include_str!("../__snapshots__/codegen/minimal__runtime.axios.ts")
    );
    // types/client/hooks/index are backend-invariant: same as the fetch goldens.
    assert_eq!(
        file(&out, "types.ts"),
        include_str!("../__snapshots__/codegen/minimal__types.ts")
    );
    assert_eq!(
        file(&out, "client.ts"),
        include_str!("../__snapshots__/codegen/minimal__client.ts")
    );
    assert_eq!(
        file(&out, "hooks.ts"),
        include_str!("../__snapshots__/codegen/minimal__hooks.ts")
    );
    assert_eq!(
        file(&out, "index.ts"),
        include_str!("../__snapshots__/codegen/minimal__index.ts")
    );
}

#[test]
fn generation_is_deterministic() {
    let spec = include_str!("../fixtures/codegen/minimal.json");
    let a = generate(spec, &full_opts()).unwrap();
    let b = generate(spec, &full_opts()).unwrap();
    for (fa, fb) in a.files.iter().zip(b.files.iter()) {
        assert_eq!(fa.rel_path, fb.rel_path);
        assert_eq!(fa.contents, fb.contents);
    }
}

/// OpenAPI 3.1 type-array nullability, `allOf` intersection, and enum unions.
#[test]
fn openapi_31_nullable_and_compositions() {
    let spec = include_str!("../fixtures/codegen/nullable_3_1.json");
    let out = generate(spec, &full_opts()).unwrap();
    let types = file(&out, "types.ts");
    assert!(
        types.contains("label?: string | null;"),
        "3.1 nullable union: {types}"
    );
    assert!(
        types.contains("score?: number | null;"),
        "3.1 nullable int: {types}"
    );
    assert!(
        types.contains("export type Kind = \"a\" | \"b\" | \"c\";"),
        "enum union: {types}"
    );
    assert!(
        types.contains("export type Owner = Named & { email?: string };"),
        "allOf intersection: {types}"
    );

    // 204 No Content delete: named void response + grouped data argument.
    assert!(
        types.contains("export type DeleteItemResponse = void;"),
        "void response alias: {types}"
    );
    let client = file(&out, "client.ts");
    assert!(
        client.contains("deleteItem(data: DeleteItemData): Promise<DeleteItemResponse>"),
        "grouped data + named response: {client}"
    );
}

/// OpenAPI 3.0 `nullable: true`, array query params, and `$ref` typing.
#[test]
fn openapi_30_petstore_shapes() {
    let spec = include_str!("../fixtures/codegen/petstore_3_0.json");
    let out = generate(spec, &full_opts()).unwrap();
    let types = file(&out, "types.ts");
    assert!(
        types.contains("tag?: string | null;"),
        "3.0 nullable: {types}"
    );
    assert!(
        types.contains("category?: Category;"),
        "$ref property: {types}"
    );

    // Grouped query in the named data type; named response aliasing a component.
    assert!(
        types.contains(
            "export type ListPetsData = { query?: { limit?: number; tags?: string[] } };"
        ),
        "array query param grouped in data: {types}"
    );
    assert!(
        types.contains("export type ShowPetByIdResponse = Pet;"),
        "named response aliases component: {types}"
    );

    let client = file(&out, "client.ts");
    assert!(
        client.contains("listPets(data: ListPetsData): Promise<ListPetsResponse>"),
        "grouped data signature: {client}"
    );
    assert!(
        client.contains("showPetById(data: ShowPetByIdData): Promise<ShowPetByIdResponse>"),
        "path param via named data: {client}"
    );

    let hooks = file(&out, "hooks.ts");
    assert!(
        hooks.contains(
            "useCreatePetMutation(options?: UseMutationOptions<CreatePetResponse, Error, CreatePetData>)"
        ),
        "mutation hook uses named types: {hooks}"
    );
}

/// Gated smoke: the generated types/runtime/client type-check under
/// `tsc --strict`. Skips when `tsc` is unavailable. `hooks.ts` is excluded
/// because it imports the `@tanstack/react-query` peer dependency.
#[test]
fn generated_typescript_typechecks() {
    if !tool_available("tsc") {
        eprintln!("[openapi_golden] skipping tsc smoke: `tsc` not on PATH");
        return;
    }

    let spec = include_str!("../fixtures/codegen/petstore_3_0.json");
    let out = generate(spec, &full_opts()).unwrap();

    let dir = std::env::temp_dir().join(format!("jet-codegen-tsc-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for name in ["types.ts", "runtime.ts", "client.ts"] {
        std::fs::write(dir.join(name), file(&out, name)).unwrap();
    }

    let status = Command::new("tsc")
        .args([
            "--noEmit",
            "--strict",
            "--skipLibCheck",
            "--lib",
            "es2020,dom",
            "--target",
            "es2020",
            "--moduleResolution",
            "bundler",
            "--module",
            "esnext",
            "types.ts",
            "runtime.ts",
            "client.ts",
        ])
        .current_dir(&dir)
        .status()
        .expect("spawn tsc");

    let _ = std::fs::remove_dir_all(&dir);
    assert!(status.success(), "generated TypeScript failed tsc --strict");
}

fn tool_available(tool: &str) -> bool {
    Command::new(tool)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
