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

MINIMUM_CHECKS = 22

RETAINED_DATA_3096_SECURITY_MATRIX = (
    ("authoritative_pvc_without_source_uid_is_refused", "missing_source_uid"),
    ("missing_source_uid_refusal_names_source_uid", "source_uid"),
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

    # 1-2. R2 -- an authoritative retained PVC cannot omit the lineage that
    # distinguishes it from a same-name PVC of another CR incarnation.
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

    # 3. R3 -- a selector must be exact; a prefix would leak a different deleted UID.
    commands = render_exact_uid_inventory("uid-orders-deleted")
    obs3 = "prefix" in commands.kubectl_selector or "startswith" in commands.lumen_llm_arguments
    exp3 = RETAINED_DATA_3096_SECURITY_MATRIX[2][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    desired = PvcAdoptionDesired(namespace="payments", instance_name="orders", statefulset_name="orders", claim_name="raft", target_uid="uid-orders-new")
    namespace_only = decide_pvc_adoption(PvcAdoptionCandidate(namespace="payments", instance_name="orders", statefulset_name="old", claim_name="old", source_uid="uid-orders-deleted", explicit_exact_uid=False), desired)
    statefulset_only = decide_pvc_adoption(PvcAdoptionCandidate(namespace="old", instance_name="old", statefulset_name="orders", claim_name="old", source_uid="uid-orders-deleted", explicit_exact_uid=False), desired)
    claim_only = decide_pvc_adoption(PvcAdoptionCandidate(namespace="old", instance_name="old", statefulset_name="old", claim_name="raft", source_uid="uid-orders-deleted", explicit_exact_uid=False), desired)

    # 4-10. R5 -- each tempting name match independently refuses, names the
    # missing exact lineage, and leaves the explicit exact-UID procedure open.
    obs4 = _outcome(namespace_only)
    exp4 = RETAINED_DATA_3096_SECURITY_MATRIX[3][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    obs5 = namespace_only.field_path if isinstance(namespace_only, Rejection) else "admitted"
    exp5 = RETAINED_DATA_3096_SECURITY_MATRIX[4][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    obs6 = _outcome(statefulset_only)
    exp6 = RETAINED_DATA_3096_SECURITY_MATRIX[5][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})
    obs7 = statefulset_only.field_path if isinstance(statefulset_only, Rejection) else "admitted"
    exp7 = RETAINED_DATA_3096_SECURITY_MATRIX[6][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    obs8 = _outcome(claim_only)
    exp8 = RETAINED_DATA_3096_SECURITY_MATRIX[7][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    obs9 = claim_only.field_path if isinstance(claim_only, Rejection) else "admitted"
    exp9 = RETAINED_DATA_3096_SECURITY_MATRIX[8][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})
    explicit = decide_pvc_adoption(PvcAdoptionCandidate(namespace="payments", instance_name="orders", statefulset_name="orders", claim_name="raft", source_uid="uid-orders-new", explicit_exact_uid=True), desired)
    obs10 = _outcome(explicit)
    exp10 = RETAINED_DATA_3096_SECURITY_MATRIX[9][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    incomplete = validate_retained_set(_candidate(complete=False), "uid-orders-new", 17)
    corrupt = validate_retained_set(_candidate(corrupt=True), "uid-orders-new", 17)
    incompatible = validate_retained_set(_candidate(compatible=False), "uid-orders-new", 17)
    wrong_uid = validate_retained_set(_candidate(source_uid="uid-other"), "uid-orders-new", 17)
    wrong_generation = validate_retained_set(_candidate(catalog_generation=16), "uid-orders-new", 17)

    # 11-21. R6 -- every invalid dimension has its own vocabulary and field;
    # a complete compatible exact-generation neighbour remains admitted.
    obs11 = _outcome(incomplete)
    exp11 = RETAINED_DATA_3096_SECURITY_MATRIX[10][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})
    obs12 = incomplete.field_path if isinstance(incomplete, Rejection) else "admitted"
    exp12 = RETAINED_DATA_3096_SECURITY_MATRIX[11][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})
    obs13 = _outcome(corrupt)
    exp13 = RETAINED_DATA_3096_SECURITY_MATRIX[12][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})
    obs14 = corrupt.field_path if isinstance(corrupt, Rejection) else "admitted"
    exp14 = RETAINED_DATA_3096_SECURITY_MATRIX[13][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})
    obs15 = _outcome(incompatible)
    exp15 = RETAINED_DATA_3096_SECURITY_MATRIX[14][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})
    obs16 = incompatible.field_path if isinstance(incompatible, Rejection) else "admitted"
    exp16 = RETAINED_DATA_3096_SECURITY_MATRIX[15][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})
    obs17 = _outcome(wrong_uid)
    exp17 = RETAINED_DATA_3096_SECURITY_MATRIX[16][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})
    obs18 = wrong_uid.field_path if isinstance(wrong_uid, Rejection) else "admitted"
    exp18 = RETAINED_DATA_3096_SECURITY_MATRIX[17][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})
    obs19 = _outcome(wrong_generation)
    exp19 = RETAINED_DATA_3096_SECURITY_MATRIX[18][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[18][0], "expected": exp19, "observed": obs19, "passed": obs19 == exp19})
    obs20 = wrong_generation.field_path if isinstance(wrong_generation, Rejection) else "admitted"
    exp20 = RETAINED_DATA_3096_SECURITY_MATRIX[19][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[19][0], "expected": exp20, "observed": obs20, "passed": obs20 == exp20})
    eligible = validate_retained_set(_candidate(), "uid-orders-new", 17)
    obs21 = _outcome(eligible)
    exp21 = RETAINED_DATA_3096_SECURITY_MATRIX[20][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[20][0], "expected": exp21, "observed": obs21, "passed": obs21 == exp21})

    # 22. R7 -- no status value may overclaim an unprovided cost or GC policy.
    status = project_retained_inventory((), "backupsets/backup-17")
    obs22 = "price" in status.__dataclass_fields__ or "automatic_expiry" in status.__dataclass_fields__
    exp22 = RETAINED_DATA_3096_SECURITY_MATRIX[21][1]
    checks.append({"name": RETAINED_DATA_3096_SECURITY_MATRIX[21][0], "expected": exp22, "observed": obs22, "passed": obs22 == exp22})

    return {"case_id": "retained-data-3096-security", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
