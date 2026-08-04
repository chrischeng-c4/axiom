from __future__ import annotations

from service_backup.application.policy import to_runtime_policy
from service_backup.domain.destination import Local
from service_backup.domain.policy import BackupPolicy, Retention, ScheduledBackupPolicy, is_expired, prunes_by_age
from service_backup.infrastructure.wire import is_structural, retention_to_json, scheduled_policy_schema, scheduled_policy_to_json

MINIMUM_CHECKS = 11

CRD_SAFE_SCHEDULED_POLICY_BEHAVIOR_MATRIX = (
    ("a_flat_policy_carries_a_schedule_a_destination_and_a_retention",
     ('0 3 * * *', 's3://b/p', 604800, None)),
    ("the_serialized_keys_are_camel_case_and_ordered",
     ('schedule', 'destination', 'retentionSecs')),
    ("an_absent_retention_is_omitted_rather_than_serialized_as_null",
     (('schedule', 'destination'), False)),
    ("the_serialized_body_is_the_values_it_was_given",
     {'schedule': '0 3 * * *', 'destination': 's3://b/p', 'retentionSecs': 604800}),
    ("retention_serializes_on_its_own_under_a_camel_case_key",
     ({'maxAgeSeconds': 3600}, {'maxAgeSeconds': 0}, {}, {})),
    ("a_valid_conversion_carries_every_field_through",
     ('BackupPolicy', '0 3 * * *', 'axiom', 'lumen', 604800)),
    ("an_absent_retention_converts_to_an_absent_maximum_age",
     (None, False, True)),
    ("the_default_retention_on_a_runtime_policy_prunes_nothing",
     (None, False, False)),
    ("the_generated_schema_holds_no_combinator_anywhere",
     (True, False, False, False, False)),
    ("the_generated_schema_names_its_type_and_its_required_fields",
     ('object', ('schedule', 'destination'), ('schedule', 'destination', 'retentionSecs'))),
    ("every_schema_property_declares_a_scalar_type",
     ({'type': 'string'}, {'type': 'string'}, {'type': 'integer', 'minimum': 0})),
)


def variant(value: object) -> str:
    """The name of the returned variant — the shape of an error-as-value."""
    return type(value).__name__


def verify_crd_safe_scheduled_policy_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. a flat policy carries a schedule a destination and a retention
    exp1 = CRD_SAFE_SCHEDULED_POLICY_BEHAVIOR_MATRIX[0][1]
    policy = ScheduledBackupPolicy(
        schedule="0 3 * * *", destination="s3://b/p", retention_secs=604800
        )
    obs1 = (policy.schedule, policy.destination, policy.retention_secs,
        ScheduledBackupPolicy("0 3 * * *", "s3://b").retention_secs)
    checks.append({"name": CRD_SAFE_SCHEDULED_POLICY_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. the serialized keys are camel case and ordered
    exp2 = CRD_SAFE_SCHEDULED_POLICY_BEHAVIOR_MATRIX[1][1]
    policy = ScheduledBackupPolicy(
        schedule="0 3 * * *", destination="s3://b/p", retention_secs=604800
        )
    obs2 = tuple(scheduled_policy_to_json(policy).keys())
    checks.append({"name": CRD_SAFE_SCHEDULED_POLICY_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. an absent retention is omitted rather than serialized as null
    exp3 = CRD_SAFE_SCHEDULED_POLICY_BEHAVIOR_MATRIX[2][1]
    bare = ScheduledBackupPolicy("0 3 * * *", "s3://b")
    obs3 = (tuple(scheduled_policy_to_json(bare).keys()),
        "retentionSecs" in scheduled_policy_to_json(bare))
    checks.append({"name": CRD_SAFE_SCHEDULED_POLICY_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. the serialized body is the values it was given
    exp4 = CRD_SAFE_SCHEDULED_POLICY_BEHAVIOR_MATRIX[3][1]
    policy = ScheduledBackupPolicy(
        schedule="0 3 * * *", destination="s3://b/p", retention_secs=604800
        )
    obs4 = scheduled_policy_to_json(policy)
    checks.append({"name": CRD_SAFE_SCHEDULED_POLICY_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. retention serializes on its own under a camel case key
    exp5 = CRD_SAFE_SCHEDULED_POLICY_BEHAVIOR_MATRIX[4][1]
    obs5 = (retention_to_json(Retention(3600)), retention_to_json(Retention(0)),
        retention_to_json(Retention(None)), retention_to_json(Retention()))
    checks.append({"name": CRD_SAFE_SCHEDULED_POLICY_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. a valid conversion carries every field through
    exp6 = CRD_SAFE_SCHEDULED_POLICY_BEHAVIOR_MATRIX[5][1]
    runtime = to_runtime_policy(
        ScheduledBackupPolicy("0 3 * * *", "s3://axiom/lumen", 604800)
        )
    obs6 = (variant(runtime), runtime.schedule, runtime.destination.bucket,
        runtime.destination.prefix, runtime.retention.max_age_seconds)
    checks.append({"name": CRD_SAFE_SCHEDULED_POLICY_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. an absent retention converts to an absent maximum age
    exp7 = CRD_SAFE_SCHEDULED_POLICY_BEHAVIOR_MATRIX[6][1]
    no_retention = to_runtime_policy(ScheduledBackupPolicy("0 3 * * *", "s3://axiom/lumen"))
    obs7 = (no_retention.retention.max_age_seconds,
        prunes_by_age(no_retention.retention), prunes_by_age(Retention(0)))
    checks.append({"name": CRD_SAFE_SCHEDULED_POLICY_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. the default retention on a runtime policy prunes nothing
    exp8 = CRD_SAFE_SCHEDULED_POLICY_BEHAVIOR_MATRIX[7][1]
    default_runtime = BackupPolicy(schedule="0 3 * * *", destination=Local("/x"))
    obs8 = (default_runtime.retention.max_age_seconds,
        prunes_by_age(default_runtime.retention),
        is_expired(0, 10**9, default_runtime.retention))
    checks.append({"name": CRD_SAFE_SCHEDULED_POLICY_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. the generated schema holds no combinator anywhere
    exp9 = CRD_SAFE_SCHEDULED_POLICY_BEHAVIOR_MATRIX[8][1]
    schema = scheduled_policy_schema()
    obs9 = (is_structural(schema), is_structural({"a": {"oneOf": []}}),
        is_structural({"a": {"anyOf": []}}), is_structural({"a": {"allOf": []}}),
        is_structural({"a": [{"oneOf": []}]}))
    checks.append({"name": CRD_SAFE_SCHEDULED_POLICY_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. the generated schema names its type and its required fields
    exp10 = CRD_SAFE_SCHEDULED_POLICY_BEHAVIOR_MATRIX[9][1]
    schema = scheduled_policy_schema()
    obs10 = (schema["type"], tuple(schema["required"]),
        tuple(schema["properties"].keys()))
    checks.append({"name": CRD_SAFE_SCHEDULED_POLICY_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. every schema property declares a scalar type
    exp11 = CRD_SAFE_SCHEDULED_POLICY_BEHAVIOR_MATRIX[10][1]
    schema = scheduled_policy_schema()
    obs11 = (schema["properties"]["schedule"], schema["properties"]["destination"],
        schema["properties"]["retentionSecs"])
    checks.append({"name": CRD_SAFE_SCHEDULED_POLICY_BEHAVIOR_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    return {
        "case_id": "crd-safe-scheduled-policy-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
