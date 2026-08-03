"""Tech design for WI #3344: aw: repair stale Python-EC fixture evidence for capability reports.

@spec #3344
"""

from __future__ import annotations

__aw_artifact_id__ = "artifact:capability-control-plane/repair-stale-python-ec-fixture-evidence-for-capability-reports-wi-3344"
__aw_work_item__ = "3344"

__aw_changes__ = """
## Changes

```yaml
changes:
  - path: apps/agentic-workflow/external-contracts/src/cases/capability-control-plane-one-way-wi-reference-direction.py
    action: modify
    section: _write_fixture / verify
    impl_mode: hand-written
    description: >
      Chosen approach: pre-computed source digest written at fixture-setup time
      (same as the scoped-dependency approach). Rationale: _write_fixture is
      called inside each Part A/B/C tempdir before _report(); AW_PYTHON_EC_SOURCE_DIGEST
      is not in scope at test time. In _write_fixture, after writing all
      src/**/*.py files, compute source_digest inline with the reference algorithm
      (sha256 over sorted ec_root/src/**/*.py, each entry: relative_path_bytes +
      NUL + len(body).to_bytes(8,'big') + NUL + body, prefix 'sha256:'), then
      compute implementation_digest = sha256(case_file.read_bytes()), and write
      evidence/claim.json with all canonical fields: protocol
      "aw.python-ec.evidence.v1", case_id "demo-coverage", mode "behavior",
      source_digest, declared_command "true" (matching pyproject.toml
      command="true" for case demo-coverage), implementation
      "src/cases/claim.py", implementation_digest, exit_code 0,
      assertions ["claim is externally observable"],
      attempts [{"exit_code":0,"assertion_count":1}].
      The stub runner 'print("fixture runner")' can remain unchanged; the
      evidence is pre-written, not runner-produced. No pyproject.toml change.
      In verify(): (1) Negative falsifier — inside a fresh tempdir call
      _write_fixture then overwrite evidence/claim.json with the old stub
      {"protocol":"aw.python-ec.evidence.v1","exit_code":0}; run _report();
      assert report["python_artifact"]["ready"] is False and
      report["next_action"]["kind"] == "run_verify". (2) Part A — inside a
      second fresh tempdir call _write_fixture (canonical evidence pre-written),
      run _report(), assert report["python_artifact"]["ready"] is True, then
      proceed with the existing create_wi assertion for wi_cell="-". Parts B
      and C follow the same canonical-evidence-first pattern, each calling
      _write_fixture inside their own tempdir.

  - path: apps/agentic-workflow/external-contracts/src/cases/capability-scoped-dependency-verification.py
    action: modify
    section: _write_python_artifacts / verify
    impl_mode: hand-written
    description: >
      In _write_python_artifacts, after writing all case source files into
      ec_root/src/cases/, compute source_digest inline with the reference
      algorithm (sha256 over sorted(ec_root/src/**/*.py), same exclusion set:
      __pycache__, .venv, venv, .pytest_cache, .mypy_cache, .ruff_cache, .tox,
      build, dist, .eggs). For each capability_id in CAPABILITY_IDS write
      evidence/<capability_id>.json with all canonical fields: protocol
      "aw.python-ec.evidence.v1", case_id "<capability_id>-behavior",
      mode "behavior", source_digest (just computed), declared_command "true"
      (matching pyproject.toml command="true" for each case), implementation
      "src/cases/<capability_id>.py",
      implementation_digest sha256(case_file.read_bytes()), exit_code 0,
      assertions ["<capability_id> fixture"],
      attempts [{"exit_code":0,"assertion_count":1}].
      No pyproject.toml or runner change; evidence_paths already point to
      evidence/<capability_id>.json. In verify(): call _write_python_artifacts(tmp)
      to establish the complete canonical TD/EC inventory and canonical evidence,
      then overwrite only evidence/leaf.json with the legacy stub
      {"status":"passed"} to begin the negative falsifier. Invoke the
      full-project unselected public command _run(tmp, cap_path, None), which
      runs `aw capability check --project demo --cap-path ... --verify
      --skip-issue-inventory` (no --capability selector); assert
      completed.returncode == 1; parse the last JSON line and assert
      json_report["python_artifact"]["ready"] is False,
      json_report["next_action"]["kind"] == "run_verify", and that the exact
      blocker string
      "Python EC case `leaf-behavior` evidence `evidence/leaf.json` has unsupported protocol"
      is a member of json_report["python_artifact"]["blockers"].
      This full-project invocation runs all fixture capability and workspace
      gates; assert all five marker files exist before proceeding, then unlink
      them. Re-call _write_python_artifacts(tmp) to restore canonical evidence
      and proceed with the existing selected-root assertions unchanged. The later
      full-project no-selector verification must assert
      json_report["python_artifact"]["ready"] is True and that the exact
      legacy-evidence blocker string is absent from
      json_report["python_artifact"]["blockers"].

  - path: apps/agentic-workflow/external-contracts/src/cases/python-td-claim-linkage.py
    action: modify
    section: _write_fixture / verify
    impl_mode: hand-written
    description: >
      Chosen approach: pre-computed source digest written at fixture-setup time
      (mirrors the one-way fix). In _write_fixture, after writing all
      ec_root/src/**/*.py files, compute source_digest inline with the reference
      algorithm, compute implementation_digest = sha256(case_file.read_bytes())
      for src/cases/claim.py, and write evidence/claim.json with all canonical
      fields: protocol "aw.python-ec.evidence.v1",
      case_id "python-td-claim-linkage", mode "behavior", source_digest,
      declared_command "true" (matching pyproject.toml command="true"),
      implementation "src/cases/claim.py", implementation_digest, exit_code 0,
      assertions ["claim is externally observable"],
      attempts [{"exit_code":0,"assertion_count":1}].
      The stub runner can remain; no pyproject.toml change. In verify():
      (1) Negative falsifier — after calling _write_fixture, overwrite
      evidence/claim.json with the old stub
      {"protocol":"aw.python-ec.evidence.v1","exit_code":0}; run _report();
      assert report["python_artifact"]["ready"] is False and
      report["next_action"]["kind"] == "run_verify" (the remediation command
      is "aw capability report --project demo --verify --skip-issue-inventory").
      This is the exact public readiness/remediation signal: when evidence is
      incomplete the report blocks on run_verify rather than exposing td_refs.
      (2) Restore canonical evidence by re-calling _write_fixture (or
      re-computing and re-writing evidence/claim.json), then run _report() and
      assert the existing td_refs and next_action.kind == "run_verify" linkage
      invariants. The negative oracle assertion is exact: both ready is False
      AND next_action.kind == "run_verify" hold for incomplete evidence;
      correct canonical evidence makes ready True and the same run_verify kind
      is expected because Status is "verified" in the capability document.
```
"""

# ---------------------------------------------------------------------------
# Design contract
# ---------------------------------------------------------------------------

def design_contract() -> str:
    """Executable design declaration required by the Python TD authoring validator."""
    target_paths = [
        "apps/agentic-workflow/external-contracts/src/cases/"
        "capability-control-plane-one-way-wi-reference-direction.py",
        "apps/agentic-workflow/external-contracts/src/cases/"
        "capability-scoped-dependency-verification.py",
        "apps/agentic-workflow/external-contracts/src/cases/"
        "python-td-claim-linkage.py",
    ]
    assert len(target_paths) == 3, target_paths

    # Unready evidence (incomplete stub) routes to run_verify.
    unready_oracle = ("run_verify", False)  # (next_action.kind, python_artifact.ready)
    assert unready_oracle == ("run_verify", False)

    # Ready evidence with no WI routes to create_wi (one-way case Part A).
    ready_no_wi_oracle = ("create_wi", True)
    assert ready_no_wi_oracle == ("create_wi", True)

    # Pre-write approach: command="true" in pyproject.toml is unchanged;
    # source_digest computed inline after source files written.
    command_field = "true"
    assert command_field == "true"

    # No production parser path in the change declaration.
    production_src = "apps/agentic-workflow/src/"
    for path in target_paths:
        assert not path.startswith(production_src), path

    return "ok"


design_details = {
    "target_paths": [
        "apps/agentic-workflow/external-contracts/src/cases/"
        "capability-control-plane-one-way-wi-reference-direction.py",
        "apps/agentic-workflow/external-contracts/src/cases/"
        "capability-scoped-dependency-verification.py",
        "apps/agentic-workflow/external-contracts/src/cases/"
        "python-td-claim-linkage.py",
    ],
    # ------------------------------------------------------------------
    # Observed seams
    # ------------------------------------------------------------------
    "fixture_seams": {
        "wi_contract_fixture.record_evidence": (
            "Writes legacy {'status':'passed',...}. Must NOT be used for positive "
            "fixtures. Observed at wi_contract_fixture.py lines 239-258."
        ),
        "canonical_pre_write_seam": (
            "The scoped-dependency case calls _write_python_artifacts() before any "
            "aw invocation. That function writes source files then immediately writes "
            "evidence. This is the correct seam for all three failing cases: compute "
            "source_digest inline after the source files are finalized, then write "
            "canonical evidence before the first public aw command. No new shared "
            "helper is needed."
        ),
        "reference_source_digest_algorithm": (
            "Observed in capability-control-plane-python-artifact-readiness.py "
            "lines 356-386: sha256 over sorted(ec_root/src/**/*.py) excluding "
            "__pycache__, .venv, venv, .pytest_cache, .mypy_cache, .ruff_cache, "
            ".tox, build, dist, .eggs. Each file contributes: "
            "relative_path_bytes + NUL + len(body).to_bytes(8,'big') + NUL + body. "
            "Prefix 'sha256:'. The fixture must reproduce this exactly so that "
            "source_digest in evidence matches what aw reads from the same tree."
        ),
        "why_not_runner_approach": (
            "command='true' in all three pyproject.toml case stanzas (observed "
            "directly). 'true' is a no-op shell command; aw ec verify runs it but "
            "it does not write evidence. A runner upgrade would also require changing "
            "the command field. The pre-write approach requires no pyproject.toml "
            "change and is consistent with how the scoped-dependency case already "
            "structures its setup."
        ),
    },
    # ------------------------------------------------------------------
    # Canonical evidence fields required by bind_evidence_record (TD #3325)
    # ------------------------------------------------------------------
    "canonical_evidence_fields": {
        "protocol": "aw.python-ec.evidence.v1",
        "case_id": "must equal [[tool.aw.python-ec.cases]] id for the evidence_paths entry",
        "mode": "behavior",
        "source_digest": "sha256 of ec_root/src/**/*.py tree computed inline after source files written",
        "declared_command": "must exactly equal the 'command' field in pyproject.toml for the case",
        "implementation": "relative path to case test_path, e.g. src/cases/claim.py",
        "implementation_digest": "sha256 of implementation file bytes at time of write",
        "exit_code": 0,
        "assertions": ["non-empty list of strings returned by the case verifier"],
        "attempts": [{"exit_code": 0, "assertion_count": "len(assertions)"}],
    },
    # ------------------------------------------------------------------
    # Per-case setup order (unambiguous, each independently valid)
    # ------------------------------------------------------------------
    "setup_order": {
        "capability-control-plane-one-way-wi-reference-direction": [
            "1. _write_fixture writes td_root, ec_root/src/**/*.py, pyproject.toml, uv.lock, unit test.",
            "2. Immediately after all src files are written, compute source_digest inline.",
            "3. Compute implementation_digest = sha256(ec_root/src/cases/claim.py bytes).",
            "4. Write ec_root/evidence/claim.json with canonical fields: "
            "case_id='demo-coverage', declared_command='true', "
            "implementation='src/cases/claim.py', assertions=['claim is externally observable'], "
            "attempts=[{'exit_code':0,'assertion_count':1}].",
            "5. _write_fixture returns. Canonical evidence is on disk.",
            "6. In verify() negative falsifier: call _write_fixture in tempdir A, then "
            "immediately overwrite evidence/claim.json with stub "
            "{'protocol':'aw.python-ec.evidence.v1','exit_code':0}; "
            "call _report(); assert python_artifact['ready'] is False "
            "and next_action['kind'] == 'run_verify'.",
            "7. Part A in tempdir B: call _write_fixture (canonical evidence written by step 4); "
            "assert python_artifact['ready'] is True; then assert next_action['kind'] == 'create_wi'.",
            "8. Parts B and C each call _write_fixture in their own tempdirs (canonical evidence present).",
        ],
        "capability-scoped-dependency-verification": [
            "1. _write_python_artifacts writes td_root source files.",
            "2. Writes ec_root/src/cases/<cap>.py for each cap in CAPABILITY_IDS.",
            "3. After all src files written, compute source_digest inline once.",
            "4. For each cap_id: compute implementation_digest = sha256(case_file bytes); "
            "write evidence/<cap_id>.json with canonical fields: "
            "case_id='<cap_id>-behavior', declared_command='true', "
            "implementation='src/cases/<cap_id>.py', "
            "assertions=['<cap_id> fixture'], attempts=[{'exit_code':0,'assertion_count':1}].",
            "5. Negative falsifier in verify(): call _write_python_artifacts(tmp) to establish "
            "the complete canonical TD/EC inventory and canonical evidence; then overwrite only "
            "evidence/leaf.json with {'status':'passed'} (legacy stub). "
            "Call _run(tmp, cap_path, None), which runs "
            "`aw capability check --project demo --cap-path ... --verify --skip-issue-inventory` "
            "(no --capability selector); assert completed.returncode == 1; parse last JSON line "
            "and assert json_report['python_artifact']['ready'] is False, "
            "json_report['next_action']['kind'] == 'run_verify', and exact blocker string "
            "'Python EC case `leaf-behavior` evidence `evidence/leaf.json` has unsupported protocol' "
            "is a member of json_report['python_artifact']['blockers']. "
            "Assert all five marker files exist, then unlink them. "
            "Re-call _write_python_artifacts(tmp) to restore canonical evidence.",
            "6. Existing selected-root _run / assertion block proceeds unchanged.",
            "7. Later full-project no-selector verification asserts "
            "json_report['python_artifact']['ready'] is True and that the exact "
            "legacy-evidence blocker string is absent from json_report['python_artifact']['blockers'].",
        ],
        "python-td-claim-linkage": [
            "1. _write_fixture writes td_root, ec_root/src/**/*.py, pyproject.toml, uv.lock, unit test.",
            "2. Immediately after all src files written, compute source_digest inline.",
            "3. Compute implementation_digest = sha256(ec_root/src/cases/claim.py bytes).",
            "4. Write ec_root/evidence/claim.json with canonical fields: "
            "case_id='python-td-claim-linkage', declared_command='true', "
            "implementation='src/cases/claim.py', assertions=['claim is externally observable'], "
            "attempts=[{'exit_code':0,'assertion_count':1}].",
            "5. _write_fixture returns. Canonical evidence on disk.",
            "6. In verify() negative falsifier: call _write_fixture, then overwrite "
            "evidence/claim.json with stub "
            "{'protocol':'aw.python-ec.evidence.v1','exit_code':0}; "
            "call _report(); assert report['python_artifact']['ready'] is False "
            "AND report['next_action']['kind'] == 'run_verify'. "
            "This is the exact public readiness/remediation signal for incomplete evidence.",
            "7. Re-write canonical evidence (re-call _write_fixture or recompute inline); "
            "call _report(); assert td_refs has one primary entry with the expected fields "
            "and next_action['kind'] == 'run_verify' (capability Status is 'verified').",
        ],
    },
    # ------------------------------------------------------------------
    # One-way case: two distinct routing states (AC3)
    # ------------------------------------------------------------------
    "one_way_routing_states": {
        "unready_evidence": {
            "condition": "evidence/claim.json is the old stub (missing case_id, source_digest, etc.)",
            "public_command": "aw capability report --project demo --include-issue-inventory",
            "exact_assertion": "report['python_artifact']['ready'] is False "
            "and report['next_action']['kind'] == 'run_verify'",
            "basis": "Frozen brief: real AW routes run_verify when Python EC readiness evidence is incomplete.",
        },
        "ready_no_wi": {
            "condition": "canonical evidence written, wi_cell='-', no tracker WI",
            "public_command": "aw capability report --project demo --include-issue-inventory",
            "exact_assertion": "report['python_artifact']['ready'] is True "
            "and report['next_action']['kind'] == 'create_wi'",
            "basis": "Existing Part A assertion is correct once readiness is established.",
        },
    },
    # ------------------------------------------------------------------
    # Negative falsifier oracles (exact public signals)
    # ------------------------------------------------------------------
    "negative_falsifiers": {
        "capability-control-plane-one-way-wi-reference-direction": {
            "evidence_written": '{"protocol":"aw.python-ec.evidence.v1","exit_code":0}',
            "public_command": "aw capability report --project demo --include-issue-inventory",
            "exact_assertions": [
                "report['python_artifact']['ready'] is False",
                "report['next_action']['kind'] == 'run_verify'",
            ],
        },
        "capability-scoped-dependency-verification": {
            "evidence_written": '{"status":"passed"} into evidence/leaf.json, after _write_python_artifacts(tmp) has established full canonical inventory',
            "public_command": "aw capability check --project demo --cap-path ... --verify --skip-issue-inventory (no --capability selector)",
            "exact_assertions": [
                "completed.returncode == 1",
                "json_report['python_artifact']['ready'] is False",
                "json_report['next_action']['kind'] == 'run_verify'",
                "'Python EC case `leaf-behavior` evidence `evidence/leaf.json` has unsupported protocol' in json_report['python_artifact']['blockers']",
                "all five marker files exist before unlink",
            ],
            "note": (
                "The full-project unselected command (_run(tmp, cap_path, None)) evaluates "
                "project-level Python artifact readiness. The scoped selected command "
                "(_run(tmp, cap_path, 'root')) intentionally omits python_artifact and "
                "can return healthy with legacy evidence — it must NOT be used for this falsifier. "
                "Canonical inventory is established first by _write_python_artifacts(tmp); only "
                "evidence/leaf.json is then overwritten with the legacy stub. "
                "After asserting the five markers, unlink them and re-call "
                "_write_python_artifacts(tmp) to restore canonical evidence before the "
                "existing selected-root assertions. The later full-project no-selector "
                "verification asserts ready is True and the exact legacy-evidence blocker "
                "string is absent."
            ),
        },
        "python-td-claim-linkage": {
            "evidence_written": '{"protocol":"aw.python-ec.evidence.v1","exit_code":0}',
            "public_command": "aw capability report --project demo --skip-issue-inventory",
            "exact_assertions": [
                "report['python_artifact']['ready'] is False",
                "report['next_action']['kind'] == 'run_verify'",
            ],
            "note": (
                "Both assertions hold simultaneously for incomplete evidence. "
                "The positive canonical-evidence run also routes to run_verify "
                "(Status=verified in the capability doc), so the distinguishing "
                "signal for the negative control is ready is False, not the kind."
            ),
        },
    },
    # ------------------------------------------------------------------
    # Production parser preservation
    # ------------------------------------------------------------------
    "production_preservation": (
        "No changes to apps/agentic-workflow/src/**. No changes to pyproject.toml, "
        "ec.lock, ec-review.json, ec-author.json, or capability documents. "
        "The three fixture case files are the only edit targets."
    ),
}
