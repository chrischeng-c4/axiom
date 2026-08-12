"""EC security case for #3096 -- fail-closed retained-data admission.

Expected values are EC-owned literals from #3096: R2 refuses incomplete
authoritative PVC evidence; R3 forbids prefix selectors; R5 refuses every
name-only PVC adoption path; R6 names each invalid retained-set condition and
keeps its eligible neighbour admitted; and R7 never projects price or automatic
expiry. Runtime storage validation and workload readiness are intentionally
outside this pure design-model contract.
"""

from __future__ import annotations

from lumen.retained_data.admission import decide_pvc_adoption, validate_retained_set
from lumen.retained_data.inventory import render_exact_uid_inventory
from lumen.retained_data.spec import BackupSetCandidate, PvcAdoptionCandidate, PvcAdoptionDesired, RetainedPvcMetadata
from lumen.retained_data.status import project_retained_inventory
from lumen.retained_data.verdict import Rejection

MINIMUM_CHECKS = 30

RETAINED_DATA_3096_SECURITY_MATRIX = (
    ("authoritative_pvc_without_source_uid_is_refused", "missing_source_uid"),
    ("missing_source_uid_refusal_names_source_uid", "source_uid"),
    ("authoritative_pvc_without_role_is_refused", "missing_role"),
    ("missing_role_refusal_names_role", "role"),
    ("authoritative_pvc_without_shard_group_is_refused", "missing_shard_group"),
    ("missing_shard_group_refusal_names_shard_group", "shard_group"),
    ("authoritative_pvc_without_format_is_refused", "missing_format"),
    ("missing_format_refusal_names_format", "format"),
    ("authoritative_pvc_without_topology_generation_is_refused", "missing_topology_generation"),
    ("missing_topology_generation_refusal_names_topology_generation", "topology_generation"),
    ("exact_uid_inventory_never_uses_prefix_selector", False),
    ("namespace_name_only_pvc_match_is_refused", "implicit_pvc_adoption"),
    ("namespace_name_refusal_names_source_uid", "source_uid"),
    ("statefulset_name_only_pvc_match_is_refused", "implicit_pvc_adoption"),
    ("statefulset_name_refusal_names_source_uid", "source_uid"),
    ("claim_name_only_pvc_match_is_refused", "implicit_pvc_adoption"),
    ("claim_name_refusal_names_source_uid", "source_uid"),
    ("exact_uid_explicit_pvc_procedure_is_admitted", "admitted"),
    ("incomplete_retained_set_is_refused", "incomplete_retained_set"),
    ("incomplete_set_refusal_names_complete", "complete"),
    ("corrupt_retained_set_is_refused", "corrupt_retained_set"),
    ("corrupt_set_refusal_names_corrupt", "corrupt"),
    ("incompatible_retained_set_is_refused", "incompatible_retained_set"),
    ("incompatible_set_refusal_names_format", "format"),
    ("wrong_uid_retained_set_is_refused", "wrong_source_uid"),
    ("wrong_uid_set_refusal_names_source_uid", "source_uid"),
    ("wrong_generation_retained_set_is_refused", "wrong_catalog_generation"),
    ("wrong_generation_set_refusal_names_catalog_generation", "catalog_generation"),
    ("eligible_complete_retained_set_is_admitted", "admitted"),
    ("retained_status_omits_price_and_automatic_expiry", False),
)


def _candidate(**overrides) -> BackupSetCandidate:
    values = {"name": "backup-17", "source_uid": "uid-orders-deleted", "catalog_generation": 17, "complete": True, "manifests_present": True, "artifacts_present": True, "compatible": True, "corrupt": False, "topology_generation": 17, "format": "raft-runtime-v1", "expected_source_uid": "uid-orders-deleted"}
    values.update(overrides)
    return BackupSetCandidate(**values)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def verify_retained_data_3096_security() -> dict:
    checks = []

    # 1-10. R2 -- an authoritative retained PVC cannot omit any immutable
    # identity dimension that distinguishes it from a same-name CR incarnation.
    try:
        RetainedPvcMetadata(source_uid="", role="authoritative", shard_group="shard-0", format="raft-runtime-v1", topology_generation=17)
    except ValueError as error:
        missing_uid_error = error
    else:
        missing_uid_error = None
    obs1 = "missing_source_uid" if missing_uid_error is not None else "admitted"
    exp1 = RETAINED_DATA_3096_SECURITY_MATRIX[0][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = "source_uid" if missing_uid_error is not None and "source_uid" in str(missing_uid_error) else "unidentified"
    exp2 = RETAINED_DATA_3096_SECURITY_MATRIX[1][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    missing_pvc_fields = (
        ("role", {"source_uid": "uid-orders-deleted", "role": "", "shard_group": "shard-0", "format": "raft-runtime-v1", "topology_generation": 17}),
        ("shard_group", {"source_uid": "uid-orders-deleted", "role": "authoritative", "shard_group": "", "format": "raft-runtime-v1", "topology_generation": 17}),
        ("format", {"source_uid": "uid-orders-deleted", "role": "authoritative", "shard_group": "shard-0", "format": "", "topology_generation": 17}),
        ("topology_generation", {"source_uid": "uid-orders-deleted", "role": "authoritative", "shard_group": "shard-0", "format": "raft-runtime-v1", "topology_generation": 0}),
    )
    missing_field_errors = {}
    for field_name, values in missing_pvc_fields:
        try:
            RetainedPvcMetadata(**values)
        except ValueError as error:
            missing_field_errors[field_name] = error
        else:
            missing_field_errors[field_name] = None

    obs3 = "missing_role" if missing_field_errors["role"] is not None else "admitted"
    exp3 = RETAINED_DATA_3096_SECURITY_MATRIX[2][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})
    obs4 = "role" if missing_field_errors["role"] is not None and "role" in str(missing_field_errors["role"]) else "unidentified"
    exp4 = RETAINED_DATA_3096_SECURITY_MATRIX[3][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    obs5 = "missing_shard_group" if missing_field_errors["shard_group"] is not None else "admitted"
    exp5 = RETAINED_DATA_3096_SECURITY_MATRIX[4][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    obs6 = "shard_group" if missing_field_errors["shard_group"] is not None and "shard_group" in str(missing_field_errors["shard_group"]) else "unidentified"
    exp6 = RETAINED_DATA_3096_SECURITY_MATRIX[5][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})
    obs7 = "missing_format" if missing_field_errors["format"] is not None else "admitted"
    exp7 = RETAINED_DATA_3096_SECURITY_MATRIX[6][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    obs8 = "format" if missing_field_errors["format"] is not None and "format" in str(missing_field_errors["format"]) else "unidentified"
    exp8 = RETAINED_DATA_3096_SECURITY_MATRIX[7][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    obs9 = "missing_topology_generation" if missing_field_errors["topology_generation"] is not None else "admitted"
    exp9 = RETAINED_DATA_3096_SECURITY_MATRIX[8][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})
    obs10 = "topology_generation" if missing_field_errors["topology_generation"] is not None and "topology_generation" in str(missing_field_errors["topology_generation"]) else "unidentified"
    exp10 = RETAINED_DATA_3096_SECURITY_MATRIX[9][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. R3 -- a selector must be exact; a prefix would leak a different deleted UID.
    commands = render_exact_uid_inventory("uid-orders-deleted")
    obs11 = "prefix" in commands.kubectl_selector or "startswith" in commands.lumen_llm_arguments
    exp11 = RETAINED_DATA_3096_SECURITY_MATRIX[10][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    desired = PvcAdoptionDesired(namespace="payments", instance_name="orders", statefulset_name="orders", claim_name="raft", target_uid="uid-orders-new")
    namespace_only = decide_pvc_adoption(PvcAdoptionCandidate(namespace="payments", instance_name="orders", statefulset_name="old", claim_name="old", source_uid="uid-orders-deleted", explicit_exact_uid=False), desired)
    statefulset_only = decide_pvc_adoption(PvcAdoptionCandidate(namespace="old", instance_name="old", statefulset_name="orders", claim_name="old", source_uid="uid-orders-deleted", explicit_exact_uid=False), desired)
    claim_only = decide_pvc_adoption(PvcAdoptionCandidate(namespace="old", instance_name="old", statefulset_name="old", claim_name="raft", source_uid="uid-orders-deleted", explicit_exact_uid=False), desired)

    # 12-18. R5 -- each tempting name match independently refuses, names the
    # missing exact lineage, and leaves the explicit exact-UID procedure open.
    obs12 = _outcome(namespace_only)
    exp12 = RETAINED_DATA_3096_SECURITY_MATRIX[11][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})
    obs13 = namespace_only.field_path if isinstance(namespace_only, Rejection) else "admitted"
    exp13 = RETAINED_DATA_3096_SECURITY_MATRIX[12][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})
    obs14 = _outcome(statefulset_only)
    exp14 = RETAINED_DATA_3096_SECURITY_MATRIX[13][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})
    obs15 = statefulset_only.field_path if isinstance(statefulset_only, Rejection) else "admitted"
    exp15 = RETAINED_DATA_3096_SECURITY_MATRIX[14][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})
    obs16 = _outcome(claim_only)
    exp16 = RETAINED_DATA_3096_SECURITY_MATRIX[15][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})
    obs17 = claim_only.field_path if isinstance(claim_only, Rejection) else "admitted"
    exp17 = RETAINED_DATA_3096_SECURITY_MATRIX[16][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})
    explicit = decide_pvc_adoption(PvcAdoptionCandidate(namespace="payments", instance_name="orders", statefulset_name="orders", claim_name="raft", source_uid="uid-orders-new", explicit_exact_uid=True), desired)
    obs18 = _outcome(explicit)
    exp18 = RETAINED_DATA_3096_SECURITY_MATRIX[17][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})

    incomplete = validate_retained_set(_candidate(complete=False), "uid-orders-new", 17)
    corrupt = validate_retained_set(_candidate(corrupt=True), "uid-orders-new", 17)
    incompatible = validate_retained_set(_candidate(compatible=False), "uid-orders-new", 17)
    wrong_uid = validate_retained_set(_candidate(source_uid="uid-other"), "uid-orders-new", 17)
    wrong_generation = validate_retained_set(_candidate(catalog_generation=16), "uid-orders-new", 17)

    # 19-29. R6 -- every invalid dimension has its own vocabulary and field;
    # a complete compatible exact-generation neighbour remains admitted.
    obs19 = _outcome(incomplete)
    exp19 = RETAINED_DATA_3096_SECURITY_MATRIX[18][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[18][0], "expected": exp19, "observed": obs19, "passed": obs19 == exp19})
    obs20 = incomplete.field_path if isinstance(incomplete, Rejection) else "admitted"
    exp20 = RETAINED_DATA_3096_SECURITY_MATRIX[19][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[19][0], "expected": exp20, "observed": obs20, "passed": obs20 == exp20})
    obs21 = _outcome(corrupt)
    exp21 = RETAINED_DATA_3096_SECURITY_MATRIX[20][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[20][0], "expected": exp21, "observed": obs21, "passed": obs21 == exp21})
    obs22 = corrupt.field_path if isinstance(corrupt, Rejection) else "admitted"
    exp22 = RETAINED_DATA_3096_SECURITY_MATRIX[21][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[21][0], "expected": exp22, "observed": obs22, "passed": obs22 == exp22})
    obs23 = _outcome(incompatible)
    exp23 = RETAINED_DATA_3096_SECURITY_MATRIX[22][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[22][0], "expected": exp23, "observed": obs23, "passed": obs23 == exp23})
    obs24 = incompatible.field_path if isinstance(incompatible, Rejection) else "admitted"
    exp24 = RETAINED_DATA_3096_SECURITY_MATRIX[23][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[23][0], "expected": exp24, "observed": obs24, "passed": obs24 == exp24})
    obs25 = _outcome(wrong_uid)
    exp25 = RETAINED_DATA_3096_SECURITY_MATRIX[24][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[24][0], "expected": exp25, "observed": obs25, "passed": obs25 == exp25})
    obs26 = wrong_uid.field_path if isinstance(wrong_uid, Rejection) else "admitted"
    exp26 = RETAINED_DATA_3096_SECURITY_MATRIX[25][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[25][0], "expected": exp26, "observed": obs26, "passed": obs26 == exp26})
    obs27 = _outcome(wrong_generation)
    exp27 = RETAINED_DATA_3096_SECURITY_MATRIX[26][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[26][0], "expected": exp27, "observed": obs27, "passed": obs27 == exp27})
    obs28 = wrong_generation.field_path if isinstance(wrong_generation, Rejection) else "admitted"
    exp28 = RETAINED_DATA_3096_SECURITY_MATRIX[27][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[27][0], "expected": exp28, "observed": obs28, "passed": obs28 == exp28})
    eligible = validate_retained_set(_candidate(), "uid-orders-new", 17)
    obs29 = _outcome(eligible)
    exp29 = RETAINED_DATA_3096_SECURITY_MATRIX[28][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[28][0], "expected": exp29, "observed": obs29, "passed": obs29 == exp29})

    # 30. R7 -- no status value may overclaim an unprovided cost or GC policy.
    status = project_retained_inventory((), "backupsets/backup-17")
    obs30 = "price" in status.__dataclass_fields__ or "automatic_expiry" in status.__dataclass_fields__
    exp30 = RETAINED_DATA_3096_SECURITY_MATRIX[29][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[29][0], "expected": exp30, "observed": obs30, "passed": obs30 == exp30})

    return {"case_id": "retained-data-3096-security", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
