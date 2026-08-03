"Tech design for WI #3339: Bind reconciliation EC oracle fixtures into digest review inputs.\n\n@spec #3339"

from __future__ import annotations


__aw_changes__ = """
changes:
  - path: apps/agentic-workflow/src/services/python_artifact.rs
    action: modify
    description: >
      Add an optional, explicit source_files declaration to the Python artifact
      protocol. Resolve each declared project-relative regular file safely,
      merge it with Python files collected from source_roots before computing
      source_digest and input_files, and cover fixture-only digest drift plus
      invalid/missing declared-file rejection without changing source-root
      Python filtering or dependency-lock digest semantics.
"""


__aw_artifact_id__ = "artifact:capability-control-plane-capability-catalog-and-td-claim-linkage/bind-reconciliation-ec-oracle-fixtures-into-digest-review-inputs-wi-3339"
__aw_work_item__ = "3339"


def design_contract() -> str:
    """Express the executable design contract for this bounded change."""

    # D1: The protocol admits an optional explicit list of project-relative
    # regular files in addition to Python source_roots. It is intentionally
    # separate from dependency_files: dependency metadata is recorded in the
    # input manifest but must not silently become source-digest authority.
    source_roots = ("src", "tests/unit")
    source_files = (
        "fixtures/claim-reconciliation/"
        "capability-catalog-td-claim-linkage-expected-mapping.json",
    )
    dependency_files = ("pyproject.toml", "uv.lock")

    assert source_roots == ("src", "tests/unit")
    assert source_files == (
        "fixtures/claim-reconciliation/"
        "capability-catalog-td-claim-linkage-expected-mapping.json",
    )
    assert dependency_files == ("pyproject.toml", "uv.lock")

    # D2: source files are canonical, safe regular project files. A missing,
    # directory, symlink, duplicate, or escaping declaration fails discovery;
    # explicit declaration is required, so a random JSON file remains ignored.
    safe_fixture = source_files[0]
    invalid_declarations = (
        "fixtures/missing.json",
        "fixtures",
        "../outside.json",
    )
    assert safe_fixture.startswith("fixtures/")
    assert all(not declaration.startswith("fixtures/claim-reconciliation/")
               for declaration in invalid_declarations)
    assert len(set(source_files)) == len(source_files)

    # D3: source_digest hashes the sorted union of Python source-root files
    # and source_files using the established relative-path/length/content
    # framing. A fixture-only mutation changes source_digest even when every
    # Python source and the dependency lock are byte-identical.
    python_inputs = (
        "src/runner.py",
        "src/cases/capability-control-plane-python-artifact-readiness.py",
    )
    source_inputs = tuple(sorted((*python_inputs, *source_files)))
    assert source_inputs == (
        "fixtures/claim-reconciliation/"
        "capability-catalog-td-claim-linkage-expected-mapping.json",
        "src/cases/capability-control-plane-python-artifact-readiness.py",
        "src/runner.py",
    )
    baseline_fixture = b'{"schema_version":"aw.python-ec.expected-mapping.v1"}\n'
    mutated_fixture = b'{"schema_version":"aw.python-ec.expected-mapping.v2"}\n'
    assert baseline_fixture != mutated_fixture
    assert python_inputs == (
        "src/runner.py",
        "src/cases/capability-control-plane-python-artifact-readiness.py",
    )

    # D4: EC lock/input-file enumeration contains the explicit fixture with a
    # content digest. This is the same bounded source authority reviewed by
    # aw ec review, not an untracked sidecar or a broad fixture-directory glob.
    expected_input_paths = tuple(sorted((*source_inputs, *dependency_files)))
    assert safe_fixture in expected_input_paths
    assert "pyproject.toml" in expected_input_paths
    assert "uv.lock" in expected_input_paths
    assert len(expected_input_paths) == len(set(expected_input_paths))

    # D5: Preserve current safe behavior: source_roots still contribute only
    # Python files and ignored cache/build directories never affect the source
    # digest. A non-Python file changes authority only if source_files names it.
    ignored_non_python = "src/build/generated.json"
    undeclared_non_python = "fixtures/unrelated.json"
    assert ignored_non_python not in source_inputs
    assert undeclared_non_python not in source_inputs

    # AC proof obligations for the Rust unit test and the public Python EC:
    # 1) discovery with safe_fixture exposes it in input_files;
    # 2) mutating only safe_fixture changes source_digest;
    # 3) stale evidence bearing the first digest is rejected after mutation;
    # 4) malformed/missing/unsafe explicit paths fail before any artifact run.
    return "ok"
