// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-tests.md#tests
// CODEGEN-BEGIN
// Byte-equivalence proof against a captured Python backend fixture.
// Hand-written until codegen can emit regenerability tests from spec
// test plans.
use agentic_workflow::generate::gen::python::{
    emit_pydantic_module, ImportIr, PydanticField, PydanticModelIr, PythonBackendSpec,
};

const SPEC_ID: &str = "python-backend-video-qc-api-models";
const SPEC_REF: &str = "tests/fixtures/python_backend/tech_design/api_models.md#models";
const EXPECTED: &str = r#""""Video QC API models.

Spec: cclab/specs/backend/workspace/bases/README.md
"""
import beanie
import pydantic

from src import bases


class VideoQCAPIRequestModel(bases.BaseAPIRequestModel):
    """VideoQCAPIRequestModel.

    Spec: cclab/specs/backend/workspace/bases/README.md

    <!-- spec-ref --> Spec: cclab/specs/backend/workspace/bases/README.md <!-- /spec-ref -->
    """
    video_url: pydantic.HttpUrl


class VideoQCPredictAPIRequestModel(bases.BaseAPIResponseModel):
    """VideoQCPredictAPIRequestModel.

    Spec: cclab/specs/backend/workspace/bases/README.md

    <!-- spec-ref --> Spec: cclab/specs/backend/workspace/bases/README.md <!-- /spec-ref -->
    """
    video_url: pydantic.HttpUrl
    blob_id: beanie.PydanticObjectId
"#;

/// @spec python-backend-emitter.md#schema
fn video_qc_spec() -> PythonBackendSpec {
    let spec_ref_line = "Spec: cclab/specs/backend/workspace/bases/README.md";
    let class_doc = |name: &str| -> String {
        format!(
            "{name}.\n\n{spec_ref_line}\n\n<!-- spec-ref --> {spec_ref_line} <!-- /spec-ref -->",
            name = name,
            spec_ref_line = spec_ref_line,
        )
    };

    PythonBackendSpec {
        spec_id: String::from(SPEC_ID),
        routers: Vec::new(),
        python_modules: Vec::new(),
        pydantic_models: vec![
            PydanticModelIr {
                name: String::from("VideoQCAPIRequestModel"),
                base: String::from("bases.BaseAPIRequestModel"),
                docstring: Some(class_doc("VideoQCAPIRequestModel")),
                fields: vec![PydanticField {
                    name: String::from("video_url"),
                    py_type: String::from("pydantic.HttpUrl"),
                    default: None,
                }],
            },
            PydanticModelIr {
                name: String::from("VideoQCPredictAPIRequestModel"),
                base: String::from("bases.BaseAPIResponseModel"),
                docstring: Some(class_doc("VideoQCPredictAPIRequestModel")),
                fields: vec![
                    PydanticField {
                        name: String::from("video_url"),
                        py_type: String::from("pydantic.HttpUrl"),
                        default: None,
                    },
                    PydanticField {
                        name: String::from("blob_id"),
                        py_type: String::from("beanie.PydanticObjectId"),
                        default: None,
                    },
                ],
            },
        ],
        imports: vec![
            ImportIr {
                module: String::from("beanie"),
                names: Vec::new(),
            },
            ImportIr {
                module: String::from("pydantic"),
                names: Vec::new(),
            },
            ImportIr {
                module: String::from("src"),
                names: vec![String::from("bases")],
            },
        ],
        module_docstring: Some(String::from(
            "Video QC API models.\n\nSpec: cclab/specs/backend/workspace/bases/README.md\n",
        )),
    }
}

/// Extract the CODEGEN-managed body from a wrapped Python file.
///
/// Returns the lines between `# CODEGEN-BEGIN` and `# CODEGEN-END`,
/// joined with `\n` and terminated by a trailing `\n` (matching the
/// `normalize()` contract). If the file has no markers, returns the
/// entire file contents.
fn extract_codegen_body(contents: &str) -> String {
    let mut in_block = false;
    let mut out = String::new();
    let mut found_marker = false;
    for line in contents.split('\n') {
        let trimmed = line.trim_start();
        if trimmed == "# CODEGEN-BEGIN" {
            in_block = true;
            found_marker = true;
            continue;
        }
        if trimmed == "# CODEGEN-END" {
            in_block = false;
            continue;
        }
        if in_block {
            out.push_str(line);
            out.push('\n');
        }
    }
    if !found_marker {
        return contents.to_string();
    }
    out
}

#[test]
fn video_qc_api_models_is_byte_equivalent() {
    let spec = video_qc_spec();
    let emitted = emit_pydantic_module(SPEC_ID, &spec, "video_qc/api_models.py");

    let disk_body = extract_codegen_body(EXPECTED);

    if emitted.content != disk_body {
        eprintln!("--- expected fixture\n+++ emitted");
        let disk_lines: Vec<&str> = disk_body.split('\n').collect();
        let emit_lines: Vec<&str> = emitted.content.split('\n').collect();
        let max = disk_lines.len().max(emit_lines.len());
        for i in 0..max {
            let d = disk_lines.get(i).copied().unwrap_or("<EOF>");
            let e = emit_lines.get(i).copied().unwrap_or("<EOF>");
            if d == e {
                eprintln!("  {:>3} {}", i + 1, d);
            } else {
                eprintln!("- {:>3} {}", i + 1, d);
                eprintln!("+ {:>3} {}", i + 1, e);
            }
        }
        panic!(
            "video_qc api_models.py byte-equivalence failed: emitted={} bytes, expected={} bytes",
            emitted.content.len(),
            disk_body.len()
        );
    }

    // Sanity: SPEC_REF documents the source-of-truth spec path.
    assert!(!SPEC_REF.is_empty());
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-tests.md#schema
// CODEGEN-BEGIN
// SPEC-REF: projects/agentic-workflow/tech-design/semantic/agentic-workflow-tests.md#schema
// TODO: Existing source behavior is covered by this feature/domain semantic TD.

// CODEGEN-END
