// CODEGEN-BEGIN
//! `openapi-codegen` — generate a typed API client from an OpenAPI 3.0, 3.1,
//! or 3.2 document, in TypeScript, Python, or Rust. Reusable polyglot codegen
//! core, extracted from `jet codegen openapi` so any CLI can compose it.
//!
//! Architecture: a language-neutral [`ir`] (document model, naming, type-name
//! map) feeds a per-language emitter under [`emit`]. The target language is
//! [`GenOptions::lang`]:
//! - [`Lang::Ts`]  → TypeScript: types + fetch/axios client + TanStack Query hooks
//! - [`Lang::Py`]  → Python: pydantic models + generated sync/async HTTP/2 runtime
//! - [`Lang::Rust`]→ Rust: serde models + reqwest client
//!
//! OpenAPI 3.2 support: the `query` path-item keyword (RFC 10008's HTTP
//! `QUERY` method) is a first-class [`ir::operations::OperationIR`] method
//! alongside `get`/`post`/etc., and `additionalOperations` entries pass
//! through without choking (parse-don't-crash for methods beyond `QUERY`).
//! Every generated `QUERY` client method also carries a POST-twin fallback
//! (epic #1296 policy): a per-client runtime option routes the call through
//! `POST` against a documented twin path instead of HTTP `QUERY` — see each
//! emitter's `client_emit` module and the crate README's "OpenAPI 3.2 and
//! HTTP QUERY" section for the exact mechanism per language. There is no
//! hard version gate: `Spec.openapi` is parsed as an opaque string, so 3.0,
//! 3.1, and 3.2 documents (and any `3.x.y` string) all parse today.
//!
//! [`generate`] is the pure core (spec text → in-memory files, no I/O); [`run`]
//! is the filesystem-writing CLI entry.

use anyhow::Result;
use std::path::{Path, PathBuf};

pub mod emit;
pub mod ir;
pub mod llm;
pub mod target;

pub use ir::{build_type_map, TypeMap};
pub use target::{
    PythonTarget, RustTarget, TargetPolicy, TargetProfile, TargetRequirements, TypeScriptTarget,
};

/// Target language for the generated client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lang {
    /// TypeScript: types + a typed fetch/axios client + TanStack Query hooks.
    #[default]
    Ts,
    /// Python: pydantic models + a generated sync/async HTTP/2 runtime.
    Py,
    /// Rust: serde models + a reqwest client.
    Rust,
}

impl Lang {
    /// Stable language identifier used in generation manifests.
    pub const fn id(self) -> &'static str {
        match self {
            Self::Ts => "typescript",
            Self::Py => "python",
            Self::Rust => "rust",
        }
    }
}

/// HTTP runtime backend for the generated TypeScript client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HttpClient {
    /// Native `fetch` (zero runtime dependency).
    #[default]
    Fetch,
    /// `axios` (peer dependency of the generated output).
    Axios,
}

/// What the generator emits, selected by CLI flags.
#[derive(Debug, Clone)]
pub struct GenOptions {
    /// Target language for the generated client.
    pub lang: Lang,
    /// Versioned language/toolchain contract. `None` preserves the legacy
    /// generated files and does not emit a target manifest.
    pub target: Option<TargetProfile>,
    pub spec_path: PathBuf,
    pub out_dir: PathBuf,
    pub client_name: String,
    pub http_client: HttpClient,
    pub emit_types: bool,
    pub emit_client: bool,
    pub emit_hooks: bool,
}

/// A single generated file, relative to the output directory.
#[derive(Debug, Clone)]
pub struct GeneratedFile {
    pub rel_path: String,
    pub contents: String,
}

/// The full in-memory generation result (so tests can assert without I/O).
#[derive(Debug, Clone)]
pub struct GeneratedOutput {
    pub files: Vec<GeneratedFile>,
    /// The explicitly selected profile used to render `files`.
    pub target: Option<TargetProfile>,
    /// The minimum requirements for consuming explicitly targeted `files`.
    pub requirements: Option<TargetRequirements>,
}

/// Sidecar filename emitted with explicitly targeted generated clients.
pub const MANIFEST_FILE: &str = ".openapi-codegen.json";

/// Stable, user-visible record of the exact generated-client contract.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GenerationManifest {
    pub schema_version: u8,
    pub generator: String,
    pub compiler: String,
    pub target: String,
    pub language: String,
    pub minimum_version: String,
    pub language_standard: String,
    pub module_system: Option<String>,
    pub module_resolution: Option<String>,
    pub strict: Option<bool>,
    pub transport: Option<String>,
    pub runtime_dependencies: Vec<String>,
}

impl GeneratedOutput {
    pub(crate) fn legacy(files: Vec<GeneratedFile>) -> Self {
        Self {
            files,
            target: None,
            requirements: None,
        }
    }

    pub(crate) fn for_target(files: Vec<GeneratedFile>, target: TargetProfile) -> Self {
        Self {
            files,
            target: Some(target),
            requirements: Some(target.requirements()),
        }
    }

    /// Build the sidecar manifest which makes the target contract inspectable
    /// after the in-memory result has been written to disk.
    pub fn manifest(&self) -> Option<GenerationManifest> {
        let requirements = self.requirements?;
        Some(GenerationManifest {
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
        })
    }

    /// Materialize generated files and, for explicit targets, their contract manifest.
    ///
    /// The output is intentionally written through this one method so every
    /// embedding CLI records the same contract rather than duplicating file
    /// loops and silently dropping target metadata.
    pub fn write_to_dir(&self, out_dir: &Path) -> Result<()> {
        let paths: Vec<PathBuf> = self
            .files
            .iter()
            .map(|file| safe_output_path(out_dir, &file.rel_path))
            .collect::<Result<_>>()?;
        std::fs::create_dir_all(out_dir)?;
        for (file, path) in self.files.iter().zip(paths) {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, &file.contents)?;
        }
        if let Some(manifest) = self.manifest() {
            let manifest = serde_json::to_string_pretty(&manifest)?;
            std::fs::write(out_dir.join(MANIFEST_FILE), format!("{manifest}\n"))?;
        }
        Ok(())
    }
}

fn safe_output_path(out_dir: &Path, rel_path: &str) -> Result<PathBuf> {
    use std::path::Component;

    let rel = Path::new(rel_path);
    if rel.is_absolute()
        || rel
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::RootDir))
    {
        anyhow::bail!("generated file path must stay under output directory: {rel_path:?}");
    }
    Ok(out_dir.join(rel))
}

impl Default for GeneratedOutput {
    fn default() -> Self {
        Self::legacy(Vec::new())
    }
}

/// Pure core: spec JSON text → generated files. No filesystem access. Dispatches
/// to the per-language emitter selected by [`GenOptions::lang`].
pub fn generate(spec_json: &str, opts: &GenOptions) -> Result<GeneratedOutput> {
    match opts.target {
        Some(target) => generate_for_target(spec_json, opts, target),
        None => match opts.lang {
            Lang::Ts => emit::ts::generate(spec_json, opts),
            Lang::Py => emit::py::generate(spec_json, opts),
            Lang::Rust => emit::rust::generate(spec_json, opts),
        },
    }
}

/// Pure core with an explicit versioned target profile. The profile must match
/// [`GenOptions::lang`], so an invalid cross-language request fails before any
/// language-specific parsing or generation occurs.
pub fn generate_for_target(
    spec_json: &str,
    opts: &GenOptions,
    target: TargetProfile,
) -> Result<GeneratedOutput> {
    if opts.lang != target.lang() {
        anyhow::bail!(
            "target profile {} is for {:?}, not requested language {:?}",
            target.id(),
            target.lang(),
            opts.lang
        );
    }
    if let Some(configured) = opts.target {
        if configured != target {
            anyhow::bail!(
                "explicit target argument {} conflicts with GenOptions target {}",
                target.id(),
                configured.id()
            );
        }
    }
    match target {
        TargetProfile::TypeScript(target) => emit::ts::generate_for_target(spec_json, opts, target),
        TargetProfile::Python(target) => emit::py::generate_for_target(spec_json, opts, target),
        TargetProfile::Rust(target) => emit::rust::generate_for_target(spec_json, opts, target),
    }
}

/// CLI entry: read spec, generate, write files. Returns a process exit code
/// (0 ok, 1 generation/write error, 2 spec read error).
pub fn run(opts: &GenOptions) -> i32 {
    let spec_json = match std::fs::read_to_string(&opts.spec_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "openapi-codegen: cannot read {}: {e}",
                opts.spec_path.display()
            );
            return 2;
        }
    };
    let output = match generate(&spec_json, opts) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("openapi-codegen: {e:#}");
            return 1;
        }
    };
    if let Err(e) = output.write_to_dir(&opts.out_dir) {
        eprintln!(
            "openapi-codegen: cannot write generated output to {}: {e}",
            opts.out_dir.display()
        );
        return 1;
    }
    for file in &output.files {
        println!("generated {}", opts.out_dir.join(&file.rel_path).display());
    }
    if output.target.is_some() {
        println!("generated {}", opts.out_dir.join(MANIFEST_FILE).display());
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn full_opts() -> GenOptions {
        GenOptions {
            lang: Lang::Ts,
            target: None,
            spec_path: PathBuf::new(),
            out_dir: PathBuf::new(),
            client_name: "createClient".to_string(),
            http_client: HttpClient::Fetch,
            emit_types: true,
            emit_client: true,
            emit_hooks: true,
        }
    }

    const MINIMAL: &str = r##"{
      "openapi": "3.0.0",
      "info": { "title": "Mini", "version": "1.0.0" },
      "paths": {
        "/pets": {
          "get": {
            "operationId": "listPets",
            "responses": { "200": { "content": { "application/json": {
              "schema": { "type": "array", "items": { "$ref": "#/components/schemas/Pet" } } } } } }
          }
        }
      },
      "components": { "schemas": {
        "Pet": { "type": "object", "properties": { "id": { "type": "integer" }, "name": { "type": "string" } }, "required": ["id", "name"] }
      } }
    }"##;

    const TARGET_PROFILE_SPEC: &str = r##"{
      "openapi": "3.0.0",
      "info": { "title": "Profiles", "version": "1.0.0" },
      "paths": {},
      "components": { "schemas": {
        "Label": { "type": "string" },
        "Pet": { "type": "object", "properties": { "gen": { "type": "string" } }, "required": ["gen"] }
      } }
    }"##;

    #[test]
    fn generates_all_files() {
        let out = generate(MINIMAL, &full_opts()).unwrap();
        let names: Vec<&str> = out.files.iter().map(|f| f.rel_path.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "types.ts",
                "runtime.ts",
                "client.ts",
                "hooks.ts",
                "index.ts"
            ]
        );
    }

    #[test]
    fn types_only_skips_client_and_hooks() {
        let mut opts = full_opts();
        opts.emit_client = false;
        opts.emit_hooks = false;
        let out = generate(MINIMAL, &opts).unwrap();
        let names: Vec<&str> = out.files.iter().map(|f| f.rel_path.as_str()).collect();
        assert_eq!(names, vec!["types.ts", "index.ts"]);
    }

    #[test]
    fn deterministic_across_runs() {
        let a = generate(MINIMAL, &full_opts()).unwrap();
        let b = generate(MINIMAL, &full_opts()).unwrap();
        for (fa, fb) in a.files.iter().zip(b.files.iter()) {
            assert_eq!(fa.rel_path, fb.rel_path);
            assert_eq!(fa.contents, fb.contents);
        }
    }

    #[test]
    fn invalid_spec_is_an_error() {
        assert!(generate("{ not json", &full_opts()).is_err());
    }

    #[test]
    fn every_lang_generates_non_empty() {
        for lang in [Lang::Ts, Lang::Py, Lang::Rust] {
            let mut opts = full_opts();
            opts.lang = lang;
            let out = generate(MINIMAL, &opts).expect("emitter runs");
            assert!(!out.files.is_empty(), "{lang:?} produced no files");
        }
    }

    #[test]
    fn python_profiles_record_requirements_and_use_their_supported_typing_syntax() {
        let mut opts = full_opts();
        opts.lang = Lang::Py;
        opts.emit_hooks = false;
        for (target, version, alias) in [
            (PythonTarget::Py311, "3.11", "Label = str"),
            (PythonTarget::Py312, "3.12", "type Label = str"),
            (PythonTarget::Py313, "3.13", "type Label = str"),
            (PythonTarget::Py314, "3.14", "type Label = str"),
        ] {
            let out =
                generate_for_target(TARGET_PROFILE_SPEC, &opts, TargetProfile::Python(target))
                    .unwrap();

            assert_eq!(out.target, Some(TargetProfile::Python(target)));
            let requirements = out.requirements.expect("profile requirements");
            assert_eq!(requirements.minimum_version, version);
            assert_eq!(requirements.runtime_dependencies, &["pydantic>=2"]);
            assert!(content(&out, "models.py").contains(alias));
            assert!(!content(&out, "models.py").contains("Optional["));
        }
    }

    #[test]
    fn materialized_output_writes_a_versioned_contract_manifest() {
        let mut opts = full_opts();
        opts.lang = Lang::Py;
        opts.emit_hooks = false;
        let output = generate_for_target(
            TARGET_PROFILE_SPEC,
            &opts,
            TargetProfile::Python(PythonTarget::Py314),
        )
        .unwrap();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "openapi-codegen-manifest-{}-{nonce}",
            std::process::id()
        ));

        output.write_to_dir(&dir).unwrap();
        let manifest: GenerationManifest =
            serde_json::from_str(&fs::read_to_string(dir.join(MANIFEST_FILE)).unwrap()).unwrap();
        assert_eq!(manifest.schema_version, 1);
        assert_eq!(manifest.target, "python-3.14");
        assert_eq!(manifest.language, "python");
        assert_eq!(manifest.minimum_version, "3.14");
        assert!(dir.join("models.py").is_file());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_escaping_output_before_writing_anything() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "openapi-codegen-prevalidate-{}-{nonce}",
            std::process::id()
        ));
        let output = GeneratedOutput::legacy(vec![
            GeneratedFile {
                rel_path: "safe.ts".to_string(),
                contents: "export const safe = true;\n".to_string(),
            },
            GeneratedFile {
                rel_path: "../escape.ts".to_string(),
                contents: "should never be written\n".to_string(),
            },
        ]);

        let error = output.write_to_dir(&dir).unwrap_err();
        assert_eq!(
            error.to_string(),
            "generated file path must stay under output directory: \"../escape.ts\""
        );
        assert!(!dir.exists());
    }

    #[test]
    fn python_profiles_compile_with_each_available_target_interpreter() {
        let mut opts = full_opts();
        opts.lang = Lang::Py;
        opts.emit_hooks = false;

        for (interpreter, target) in [
            ("python3.11", PythonTarget::Py311),
            ("python3.12", PythonTarget::Py312),
            ("python3.13", PythonTarget::Py313),
            ("python3.14", PythonTarget::Py314),
        ] {
            let available = Command::new(interpreter)
                .arg("--version")
                .output()
                .is_ok_and(|status| status.status.success());
            if !available {
                continue;
            }
            let output =
                generate_for_target(TARGET_PROFILE_SPEC, &opts, TargetProfile::Python(target))
                    .unwrap();
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let dir = std::env::temp_dir().join(format!(
                "openapi-codegen-profile-{}-{nonce}",
                std::process::id()
            ));
            fs::create_dir_all(&dir).unwrap();
            let mut paths = Vec::new();
            for file in &output.files {
                let path = dir.join(&file.rel_path);
                fs::write(&path, &file.contents).unwrap();
                paths.push(path);
            }
            let result = Command::new(interpreter)
                .arg("-m")
                .arg("py_compile")
                .args(&paths)
                .output()
                .unwrap();
            assert!(
                result.status.success(),
                "{interpreter} cannot compile {} output\n{}",
                output.target.expect("profile target").id(),
                String::from_utf8_lossy(&result.stderr)
            );
            let _ = fs::remove_dir_all(dir);
        }
    }

    #[test]
    fn rust_2024_profile_escapes_gen_field_without_changing_rust_2021() {
        let mut opts = full_opts();
        opts.lang = Lang::Rust;
        opts.emit_hooks = false;
        let rust_2021 = generate_for_target(
            TARGET_PROFILE_SPEC,
            &opts,
            TargetProfile::Rust(RustTarget::Rust2021),
        )
        .unwrap();
        let rust_2024 = generate_for_target(
            TARGET_PROFILE_SPEC,
            &opts,
            TargetProfile::Rust(RustTarget::Rust2024),
        )
        .unwrap();

        assert!(content(&rust_2021, "models.rs").contains("pub gen: String,"));
        assert!(content(&rust_2024, "models.rs").contains("pub gen_: String,"));
        assert!(content(&rust_2024, "models.rs").contains("#[serde(rename = \"gen\")]"));
        assert_eq!(
            rust_2024
                .requirements
                .expect("profile requirements")
                .minimum_version,
            "1.85"
        );
    }

    #[test]
    fn target_profile_must_match_the_requested_language() {
        let error = generate_for_target(
            MINIMAL,
            &full_opts(),
            TargetProfile::Python(PythonTarget::Py311),
        )
        .unwrap_err();
        assert!(error.to_string().contains("python-3.11"));
    }

    #[test]
    fn custom_client_name() {
        let mut opts = full_opts();
        opts.client_name = "makeApi".to_string();
        let out = generate(MINIMAL, &opts).unwrap();
        let client = out
            .files
            .iter()
            .find(|f| f.rel_path == "client.ts")
            .unwrap();
        assert!(client
            .contents
            .contains("export function makeApi(config: ClientConfig)"));
        assert!(client.contents.contains("ReturnType<typeof makeApi>"));
    }

    fn content<'a>(out: &'a GeneratedOutput, name: &str) -> &'a str {
        out.files
            .iter()
            .find(|f| f.rel_path == name)
            .unwrap()
            .contents
            .as_str()
    }

    #[test]
    fn http_backend_only_changes_runtime() {
        let fetch = generate(MINIMAL, &full_opts()).unwrap();
        let mut axios_opts = full_opts();
        axios_opts.http_client = HttpClient::Axios;
        let axios = generate(MINIMAL, &axios_opts).unwrap();

        // Everything except runtime.ts is byte-identical across backends.
        for name in ["types.ts", "client.ts", "hooks.ts", "index.ts"] {
            assert_eq!(
                content(&fetch, name),
                content(&axios, name),
                "{name} differs across backends"
            );
        }

        // The fetch runtime uses native fetch; the axios runtime imports axios.
        let fetch_rt = content(&fetch, "runtime.ts");
        assert!(fetch_rt.contains("const doFetch = config.fetch ?? fetch;"));
        assert!(!fetch_rt.contains("axios"));
        let axios_rt = content(&axios, "runtime.ts");
        assert!(axios_rt.contains("import axios from \"axios\";"));
        assert!(axios_rt.contains("axios?: AxiosInstance;"));
        assert!(axios_rt.contains("config.axios ?? axios.create()"));
    }
}
// CODEGEN-END
