from __future__ import annotations

from service_backup.application.runner import object_key, plan_backup_run, plan_prune, run_result_to_json
from service_backup.application.sink import SinkKind
from service_backup.domain.policy import Retention
from service_backup.infrastructure.keys import build_key, list_prefix, local_object_name, normalize_prefix

MINIMUM_CHECKS = 12

RETAINED_OBJECT_WRITE_BEHAVIOR_MATRIX = (
    ("a_run_reports_the_sink_the_key_the_size_and_the_prune_count",
     ('s3://axiom/lumen', 'lumen/backup-1767225600.json', 4096, 1767225600, 0)),
    ("the_wire_summary_uses_camel_case_keys",
     (('object', 'pruned'), ('sink', 'key', 'bytes', 'unixSeconds'))),
    ("the_wire_summary_carries_the_same_values_the_plan_reported",
     {'object': {'sink': 's3://axiom/lumen', 'key': 'lumen/backup-1767225600.json', 'bytes': 10, 'unixSeconds': 1767225600}, 'pruned': 2}),
    ("an_absent_maximum_age_prunes_nothing",
     (0, ())),
    ("a_finite_maximum_age_selects_only_what_is_older_than_the_cutoff",
     ('lumen/backup-1767139200.json',)),
    ("the_expiry_comparison_is_strict_on_the_cutoff_second",
     ((), ('lumen/backup-1767225600.json',), 1)),
    ("an_object_this_crate_did_not_write_is_left_alone",
     ('lumen/backup-1.json',)),
    ("the_prefix_used_for_pruning_is_normalized_first",
     (('lumen/backup-1.json',), ('lumen/backup-1.json',), 'lumen', 'lumen', '')),
    ("a_local_key_and_an_object_store_key_have_different_shapes",
     ('backup-1767225600.json', 'lumen/backup-1767225600.json', 'lumen/backup-1767225600.json', 'backup-1767225600.json')),
    ("a_sink_kind_with_no_key_shape_is_refused_and_a_raw_prefix_is_normalized",
     ('ValueError', 'lumen/backup-1767225600.json')),
    ("a_key_carries_the_timestamp_it_was_built_from",
     ('lumen/backup-1767225600.json', 'backup-1767225600.json', 'a/b/backup-1767225600.json', 'backup-1767225600.json')),
    ("the_listing_prefix_ends_at_a_path_segment_boundary",
     ('lumen/', None, 'a/b/')),
)


def refusal(fn, *args, **kwargs) -> str:
    """The exception type a refusing call raises, or 'accepted' if it returns."""
    try:
        fn(*args, **kwargs)
    except Exception as exc:  # noqa: BLE001 — the type is the observation
        return type(exc).__name__
    return "accepted"


TS = 1767225600


def verify_retained_object_write_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. a run reports the sink the key the size and the prune count
    exp1 = RETAINED_OBJECT_WRITE_BEHAVIOR_MATRIX[0][1]
    result = plan_backup_run("s3://axiom/lumen", "lumen", SinkKind.S3, 4096, TS,
        Retention(None))
    obs1 = (result.object.sink, result.object.key, result.object.bytes,
        result.object.unix_seconds, result.pruned)
    checks.append({"name": RETAINED_OBJECT_WRITE_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. the wire summary uses camel case keys
    exp2 = RETAINED_OBJECT_WRITE_BEHAVIOR_MATRIX[1][1]
    result = plan_backup_run("s3://axiom/lumen", "lumen", SinkKind.S3, 4096, TS,
        Retention(None))
    obs2 = (tuple(run_result_to_json(result).keys()),
        tuple(run_result_to_json(result)["object"].keys()))
    checks.append({"name": RETAINED_OBJECT_WRITE_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. the wire summary carries the same values the plan reported
    exp3 = RETAINED_OBJECT_WRITE_BEHAVIOR_MATRIX[2][1]
    pruning = plan_backup_run("s3://axiom/lumen", "lumen", SinkKind.S3, 10, TS,
        Retention(0),
        ("lumen/backup-1.json", "lumen/backup-2.json"))
    obs3 = run_result_to_json(pruning)
    checks.append({"name": RETAINED_OBJECT_WRITE_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. an absent maximum age prunes nothing
    exp4 = RETAINED_OBJECT_WRITE_BEHAVIOR_MATRIX[3][1]
    unlimited = plan_backup_run("s3://axiom/lumen", "lumen", SinkKind.S3, 10, TS,
        Retention(None),
        ("lumen/backup-1.json", "lumen/backup-2.json"))
    obs4 = (unlimited.pruned,
        plan_prune(("lumen/backup-1.json",), "lumen", TS, Retention(None)))
    checks.append({"name": RETAINED_OBJECT_WRITE_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. a finite maximum age selects only what is older than the cutoff
    exp5 = RETAINED_OBJECT_WRITE_BEHAVIOR_MATRIX[4][1]
    aged = ("lumen/backup-1767139200.json", "lumen/backup-1767225599.json",
        "lumen/backup-1767225600.json")
    obs5 = plan_prune(aged, "lumen", TS, Retention(3600))
    checks.append({"name": RETAINED_OBJECT_WRITE_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. the expiry comparison is strict on the cutoff second
    exp6 = RETAINED_OBJECT_WRITE_BEHAVIOR_MATRIX[5][1]
    obs6 = (plan_prune(("lumen/backup-1767225600.json",), "lumen", TS, Retention(0)),
        plan_prune(("lumen/backup-1767225600.json",), "lumen", TS + 1, Retention(0)),
        plan_backup_run("s3://axiom/lumen", "lumen", SinkKind.S3, 10, TS,
        Retention(0), ("lumen/backup-1767225599.json",)).pruned)
    checks.append({"name": RETAINED_OBJECT_WRITE_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. an object this crate did not write is left alone
    exp7 = RETAINED_OBJECT_WRITE_BEHAVIOR_MATRIX[6][1]
    obs7 = plan_prune(("lumen/foreign.json", "lumen/backup-1.json",
        "other/backup-1.json", "lumen/backup-1.txt"),
        "lumen", TS, Retention(0))
    checks.append({"name": RETAINED_OBJECT_WRITE_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. the prefix used for pruning is normalized first
    exp8 = RETAINED_OBJECT_WRITE_BEHAVIOR_MATRIX[7][1]
    obs8 = (plan_prune(("lumen/backup-1.json",), "/lumen/", TS, Retention(0)),
        plan_prune(("lumen/backup-1.json",), "lumen", TS, Retention(0)),
        normalize_prefix("/lumen/"), normalize_prefix("lumen"), normalize_prefix(""))
    checks.append({"name": RETAINED_OBJECT_WRITE_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. a local key and an object store key have different shapes
    exp9 = RETAINED_OBJECT_WRITE_BEHAVIOR_MATRIX[8][1]
    obs9 = (object_key(SinkKind.LOCAL, "backup", TS), object_key(SinkKind.S3, "lumen", TS),
        object_key(SinkKind.GCS, "lumen", TS), object_key(SinkKind.S3, "", TS))
    checks.append({"name": RETAINED_OBJECT_WRITE_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. a sink kind with no key shape is refused and a raw prefix is normalized
    exp10 = RETAINED_OBJECT_WRITE_BEHAVIOR_MATRIX[9][1]
    obs10 = (refusal(object_key, SinkKind.UNSUPPORTED_CLOUD, "lumen", TS),
        object_key(SinkKind.S3, "/lumen/", TS))
    checks.append({"name": RETAINED_OBJECT_WRITE_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. a key carries the timestamp it was built from
    exp11 = RETAINED_OBJECT_WRITE_BEHAVIOR_MATRIX[10][1]
    obs11 = (build_key("lumen", TS), build_key("", TS), build_key("a/b", TS),
        local_object_name("backup", TS))
    checks.append({"name": RETAINED_OBJECT_WRITE_BEHAVIOR_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. the listing prefix ends at a path segment boundary
    exp12 = RETAINED_OBJECT_WRITE_BEHAVIOR_MATRIX[11][1]
    obs12 = (list_prefix("lumen"), list_prefix(""), list_prefix("a/b"))
    checks.append({"name": RETAINED_OBJECT_WRITE_BEHAVIOR_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    return {
        "case_id": "retained-object-write-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
