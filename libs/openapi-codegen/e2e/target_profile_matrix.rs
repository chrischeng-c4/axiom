// <HANDWRITE gap="missing-generator:external-target-profile-matrix" tracker="#1569" reason="external compiler and runtime oracles require process orchestration">
// @spec libs/openapi-codegen/external-contracts/behavior/multi-language-openapi-client-generation-contract.md#external-contract

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use openapi_codegen::{
    generate, GenOptions, GeneratedOutput, GenerationManifest, HttpClient, Lang, PythonTarget,
    RustTarget, TargetProfile, TargetRequirements, TypeScriptTarget, MANIFEST_FILE,
};

const PROFILE_SPEC: &str = r##"{
  "openapi": "3.1.0",
  "info": { "title": "Target profile contract", "version": "1.0.0" },
  "paths": {},
  "components": { "schemas": {
    "Label": { "type": "string" },
    "Pet": {
      "type": "object",
      "properties": {
        "gen": { "type": "string" },
        "tag": { "type": "string" }
      },
      "required": ["gen"]
    }
  } }
}"##;

fn opts(lang: Lang, target: Option<TargetProfile>) -> GenOptions {
    GenOptions {
        lang,
        target,
        spec_path: PathBuf::new(),
        out_dir: PathBuf::new(),
        client_name: "createClient".to_string(),
        http_client: HttpClient::Fetch,
        emit_types: true,
        emit_client: true,
        emit_hooks: false,
    }
}

fn temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "openapi-codegen-{label}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("create contract temp directory");
    path
}

fn file<'a>(output: &'a GeneratedOutput, name: &str) -> &'a str {
    &output
        .files
        .iter()
        .find(|file| file.rel_path == name)
        .unwrap_or_else(|| panic!("generated file {name}"))
        .contents
}

fn command_output(command: &mut Command, description: &str) -> Output {
    command
        .output()
        .unwrap_or_else(|error| panic!("{description}: failed to spawn: {error}"))
}

fn assert_success(output: Output, description: &str) {
    assert!(
        output.status.success(),
        "{description}: exit {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

fn expected_manifest(requirements: TargetRequirements) -> GenerationManifest {
    GenerationManifest {
        schema_version: 1,
        generator: "openapi-codegen".to_string(),
        compiler: requirements.compiler.to_string(),
        target: requirements.target.to_string(),
        language: requirements.language.id().to_string(),
        minimum_version: requirements.minimum_version.to_string(),
        language_standard: requirements.language_standard.to_string(),
        module_system: requirements.module_system.map(str::to_string),
        module_resolution: requirements.module_resolution.map(str::to_string),
        strict: requirements.strict,
        transport: requirements.transport.map(str::to_string),
        runtime_dependencies: requirements
            .runtime_dependencies
            .iter()
            .map(|dependency| (*dependency).to_string())
            .collect(),
    }
}

fn assert_materialized_manifest(output: &GeneratedOutput, out_dir: &PathBuf) -> GenerationManifest {
    output
        .write_to_dir(out_dir)
        .expect("materialize targeted generated output");
    let bytes = fs::read(out_dir.join(MANIFEST_FILE)).expect("read target sidecar");
    let manifest: GenerationManifest =
        serde_json::from_slice(&bytes).expect("parse target sidecar");
    assert_eq!(
        manifest,
        expected_manifest(output.requirements.expect("target requirements"))
    );
    manifest
}

fn verify_python(target: PythonTarget, version: &str, pep695: bool) {
    let profile = TargetProfile::Python(target);
    let output = generate(PROFILE_SPEC, &opts(Lang::Py, Some(profile))).expect("generate Python");
    assert_eq!(output.target, Some(profile));
    let requirements = output.requirements.expect("Python target requirements");
    assert_eq!(requirements.minimum_version, version);
    assert_eq!(requirements.runtime_dependencies, &["pydantic>=2"]);
    assert_eq!(requirements.compiler, "python");

    let models = file(&output, "models.py");
    if pep695 {
        assert!(models.contains("type Label = str"));
    } else {
        assert!(models.contains("Label = str"));
        assert!(!models.contains("type Label = str"));
    }
    assert!(models.contains("tag: str | None = None"));

    let root = temp_dir(&format!("python-{version}"));
    let package = root.join("generated_api");
    fs::create_dir_all(&package).expect("create generated Python package");
    let manifest = assert_materialized_manifest(&output, &package);
    assert_eq!(manifest.target, profile.id());
    assert_eq!(manifest.minimum_version, version);
    let script = r#"
import pathlib
import py_compile
import sys

root = pathlib.Path(sys.argv[1])
for path in sorted(root.glob("*.py")):
    py_compile.compile(str(path), doraise=True)
sys.path.insert(0, str(root.parent))
from generated_api import Pet
pet = Pet.model_validate({"gen": "wire-value"})
assert pet.gen == "wire-value"
assert pet.tag is None
assert pet.model_dump(by_alias=True)["gen"] == "wire-value"
"#;
    let verification = command_output(
        Command::new("uv")
            .arg("run")
            .arg("--quiet")
            .arg("--no-project")
            .arg("--python")
            .arg(version)
            .arg("--with")
            .arg("pydantic==2.12.5")
            .arg("python")
            .arg("-c")
            .arg(script)
            .arg(&package),
        &format!("Python {version} generated-model smoke"),
    );
    assert_success(
        verification,
        &format!("Python {version} generated-model smoke"),
    );
    fs::remove_dir_all(root).expect("remove Python contract temp directory");
}

#[test]
fn python_311_generated_model_contract() {
    verify_python(PythonTarget::Py311, "3.11", false);
}

#[test]
fn python_312_generated_model_contract() {
    verify_python(PythonTarget::Py312, "3.12", true);
}

#[test]
fn python_313_generated_model_contract() {
    verify_python(PythonTarget::Py313, "3.13", true);
}

#[test]
fn python_314_generated_model_contract() {
    verify_python(PythonTarget::Py314, "3.14", true);
}

#[test]
fn typescript_50_strict_modern_module_contract() {
    let profile = TargetProfile::TypeScript(TypeScriptTarget::Ts50);
    let output =
        generate(PROFILE_SPEC, &opts(Lang::Ts, Some(profile))).expect("generate TypeScript");
    assert_eq!(
        output
            .files
            .iter()
            .map(|file| file.rel_path.as_str())
            .collect::<Vec<_>>(),
        vec!["types.ts", "runtime.ts", "client.ts", "index.ts"]
    );
    let requirements = output.requirements.expect("TypeScript target requirements");
    assert_eq!(requirements.minimum_version, "5.0");
    assert_eq!(requirements.language_standard, "ES2022");
    assert_eq!(requirements.module_system, Some("ESNext"));
    assert_eq!(requirements.module_resolution, Some("Bundler"));
    assert_eq!(requirements.strict, Some(true));

    let root = temp_dir("typescript-5.0");
    let manifest = assert_materialized_manifest(&output, &root);
    assert_eq!(manifest.target, "typescript-5.0");
    fs::write(
        root.join("consumer.ts"),
        r#"import { createClient } from "./client";
import type { Pet } from "./types";

const pet: Pet = { gen: "wire-value" };
const client = createClient({ baseUrl: "https://example.invalid" });
void pet;
void client;
"#,
    )
    .expect("write independent TypeScript consumer");
    fs::write(
        root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "strict": true,
    "verbatimModuleSyntax": true,
    "noEmit": true
  },
  "include": ["*.ts"]
}
"#,
    )
    .expect("write strict TypeScript config");
    let verification = command_output(
        Command::new("npx")
            .arg("--yes")
            .arg("--package=typescript@5.0.4")
            .arg("tsc")
            .arg("--project")
            .arg("tsconfig.json")
            .current_dir(&root),
        "TypeScript 5.0 strict modern-module typecheck",
    );
    assert_success(
        verification,
        "TypeScript 5.0 strict modern-module typecheck",
    );
    fs::remove_dir_all(root).expect("remove TypeScript contract temp directory");
}

fn verify_rust(target: RustTarget, edition: &str) {
    let profile = TargetProfile::Rust(target);
    let output = generate(PROFILE_SPEC, &opts(Lang::Rust, Some(profile))).expect("generate Rust");
    assert_eq!(
        output
            .files
            .iter()
            .map(|file| file.rel_path.as_str())
            .collect::<Vec<_>>(),
        vec!["models.rs", "client.rs", "mod.rs"]
    );
    let requirements = output.requirements.expect("Rust target requirements");
    assert_eq!(requirements.language_standard, edition);
    assert_eq!(requirements.transport, Some("reqwest-blocking"));
    let models = file(&output, "models.rs");
    match target {
        RustTarget::Rust2021 => assert!(models.contains("pub gen: String,")),
        RustTarget::Rust2024 => {
            assert!(models.contains("#[serde(rename = \"gen\")]"));
            assert!(models.contains("pub gen_: String,"));
        }
    }

    let root = temp_dir(&format!("rust-{edition}"));
    let source = root.join("src");
    fs::create_dir_all(&source).expect("create generated Rust source directory");
    let manifest = assert_materialized_manifest(&output, &source);
    assert_eq!(manifest.target, profile.id());
    fs::write(
        root.join("Cargo.toml"),
        format!(
            r#"[package]
name = "openapi_target_{edition}"
version = "0.0.0"
edition = "{edition}"

[dependencies]
reqwest = {{ version = "0.12", features = ["blocking", "json"] }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
"#
        ),
    )
    .expect("write generated Rust Cargo manifest");
    fs::rename(source.join("mod.rs"), source.join("lib.rs")).expect("install generated lib root");
    let crate_name = format!("openapi_target_{edition}");
    let field = if target == RustTarget::Rust2024 {
        "gen_"
    } else {
        "gen"
    };
    let tests = root.join("tests");
    fs::create_dir_all(&tests).expect("create generated Rust consumer tests");
    fs::write(
        tests.join("consumer.rs"),
        format!(
            r#"use {crate_name}::models::Pet;

#[test]
fn generated_pet_preserves_gen_wire_contract() {{
    let pet = Pet {{ {field}: "wire-value".to_string(), tag: None }};
    let json = serde_json::to_value(&pet).expect("serialize generated Pet");
    assert_eq!(json.get("gen").and_then(|value| value.as_str()), Some("wire-value"));
    assert!(json.get("gen_").is_none());
    let decoded: Pet = serde_json::from_value(json).expect("deserialize generated Pet");
    assert_eq!(decoded.{field}, "wire-value");
}}
"#
        ),
    )
    .expect("write independent Rust consumer");
    let verification = command_output(
        Command::new("cargo")
            .arg("test")
            .arg("--quiet")
            .current_dir(&root),
        &format!("Rust {edition} generated-client consumer"),
    );
    assert_success(
        verification,
        &format!("Rust {edition} generated-client consumer"),
    );
    fs::remove_dir_all(root).expect("remove Rust contract temp directory");
}

#[test]
fn rust_2021_generated_client_contract() {
    verify_rust(RustTarget::Rust2021, "2021");
}

#[test]
fn rust_2024_generated_client_gen_property_contract() {
    verify_rust(RustTarget::Rust2024, "2024");
}

fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}

fn output_fingerprint(output: &GeneratedOutput) -> Vec<(&str, u64)> {
    output
        .files
        .iter()
        .map(|file| (file.rel_path.as_str(), fnv1a(file.contents.as_bytes())))
        .collect()
}

#[test]
fn legacy_default_output_contract() {
    let typescript = generate(PROFILE_SPEC, &opts(Lang::Ts, None)).expect("legacy TypeScript");
    let python = generate(PROFILE_SPEC, &opts(Lang::Py, None)).expect("legacy Python");
    let rust = generate(PROFILE_SPEC, &opts(Lang::Rust, None)).expect("legacy Rust");

    assert_eq!(typescript.target, None);
    assert_eq!(python.target, None);
    assert_eq!(rust.target, None);
    assert_eq!(typescript.manifest(), None);
    assert_eq!(python.manifest(), None);
    assert_eq!(rust.manifest(), None);
    assert_eq!(
        output_fingerprint(&typescript),
        vec![
            ("types.ts", 11271619169474842166),
            ("runtime.ts", 7139633933126818586),
            ("client.ts", 2626699975202142277),
            ("index.ts", 14572872411216220858),
        ]
    );
    assert_eq!(
        output_fingerprint(&python),
        vec![
            ("models.py", 16913750451280483632),
            ("h2c_runtime.py", 15121160305850173931),
            ("client.py", 10786669601410878341),
            ("__init__.py", 9994428160985782841),
        ]
    );
    assert_eq!(
        output_fingerprint(&rust),
        vec![
            ("models.rs", 9804341337094805699),
            ("client.rs", 10528869006445775057),
            ("mod.rs", 10596960125496975377),
        ]
    );

    for (label, output) in [
        ("typescript", &typescript),
        ("python", &python),
        ("rust", &rust),
    ] {
        let root = temp_dir(&format!("legacy-{label}-no-manifest"));
        output
            .write_to_dir(&root)
            .expect("materialize legacy output");
        assert!(
            !root.join(MANIFEST_FILE).exists(),
            "legacy {label} unexpectedly wrote target manifest"
        );
        fs::remove_dir_all(root).expect("remove legacy contract temp directory");
    }
}

#[test]
fn all_target_requirements_and_artifacts_are_deterministic() {
    for profile in [
        TargetProfile::Python(PythonTarget::Py311),
        TargetProfile::Python(PythonTarget::Py312),
        TargetProfile::Python(PythonTarget::Py313),
        TargetProfile::Python(PythonTarget::Py314),
        TargetProfile::TypeScript(TypeScriptTarget::Ts50),
        TargetProfile::Rust(RustTarget::Rust2021),
        TargetProfile::Rust(RustTarget::Rust2024),
    ] {
        let first = generate(PROFILE_SPEC, &opts(profile.lang(), Some(profile)))
            .expect("first targeted generation");
        let second = generate(PROFILE_SPEC, &opts(profile.lang(), Some(profile)))
            .expect("second targeted generation");
        assert_eq!(first.target, second.target);
        assert_eq!(first.requirements, second.requirements);
        assert_eq!(first.manifest(), second.manifest());
        assert_eq!(output_fingerprint(&first), output_fingerprint(&second));

        let first_dir = temp_dir(&format!("{}-first", profile.id()));
        let second_dir = temp_dir(&format!("{}-second", profile.id()));
        assert_materialized_manifest(&first, &first_dir);
        assert_materialized_manifest(&second, &second_dir);
        assert_eq!(
            fs::read(first_dir.join(MANIFEST_FILE)).expect("read first manifest bytes"),
            fs::read(second_dir.join(MANIFEST_FILE)).expect("read second manifest bytes"),
        );
        fs::remove_dir_all(first_dir).expect("remove first deterministic output");
        fs::remove_dir_all(second_dir).expect("remove second deterministic output");
    }
}

// </HANDWRITE>
