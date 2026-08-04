from __future__ import annotations

from service_backup.application.parse import parse_destination
from service_backup.application.policy import to_runtime_policy
from service_backup.domain.destination import Local
from service_backup.domain.errors import EmptySchedule, describe
from service_backup.domain.policy import BackupPolicy, Retention, ScheduledBackupPolicy, is_blank_schedule, prunes_by_age
from service_backup.infrastructure.wire import scheduled_policy_to_json

MINIMUM_CHECKS = 11

CRD_SAFE_SCHEDULED_POLICY_SECURITY_MATRIX = (
    ("an_empty_schedule_is_refused_at_admission",
     ('EmptySchedule', 'BackupPolicy')),
    ("a_whitespace_only_schedule_is_refused_too",
     ('EmptySchedule', 'EmptySchedule', 'EmptySchedule', 'EmptySchedule')),
    ("the_blankness_test_reads_the_same_way_when_called_directly",
     (True, True, True, False, False)),
    ("an_unparseable_destination_fails_at_admission",
     ('UnsupportedScheme', 'ftp://h/x', 'EmptyDestination', 'MissingBucket')),
    ("admission_reads_the_destination_with_the_writers_own_parser",
     ('S3', True, True, True)),
    ("the_schedule_is_checked_before_the_destination",
     ('EmptySchedule', 'EmptySchedule', 'UnsupportedScheme')),
    ("an_admitted_schedule_is_carried_through_byte_for_byte",
     (' 0 3 * * * ', True, 11)),
    ("an_admitted_policy_is_immutable",
     ('FrozenInstanceError', 'FrozenInstanceError', 'FrozenInstanceError')),
    ("an_admission_refusal_is_returned_rather_than_raised",
     ('accepted', 'accepted', 'EmptySchedule')),
    ("the_empty_schedule_refusal_reads_as_a_sentence",
     ('backup schedule must not be empty', 'backup schedule must not be empty')),
    ("a_zero_retention_is_a_configured_retention_not_an_absent_one",
     (0, True, {'schedule': '0 3 * * *', 'destination': 's3://b', 'retentionSecs': 0})),
)


def variant(value: object) -> str:
    """The name of the returned variant — the shape of an error-as-value."""
    return type(value).__name__


def refusal(fn, *args, **kwargs) -> str:
    """The exception type a refusing call raises, or 'accepted' if it returns."""
    try:
        fn(*args, **kwargs)
    except Exception as exc:  # noqa: BLE001 — the type is the observation
        return type(exc).__name__
    return "accepted"


def verify_crd_safe_scheduled_policy_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. an empty schedule is refused at admission
    exp1 = CRD_SAFE_SCHEDULED_POLICY_SECURITY_MATRIX[0][1]
    obs1 = (variant(to_runtime_policy(ScheduledBackupPolicy("", "s3://b/p"))),
        variant(to_runtime_policy(ScheduledBackupPolicy("0 3 * * *", "s3://b/p"))))
    checks.append({"name": CRD_SAFE_SCHEDULED_POLICY_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. a whitespace only schedule is refused too
    exp2 = CRD_SAFE_SCHEDULED_POLICY_SECURITY_MATRIX[1][1]
    obs2 = (variant(to_runtime_policy(ScheduledBackupPolicy(" ", "s3://b"))),
        variant(to_runtime_policy(ScheduledBackupPolicy("\t", "s3://b"))),
        variant(to_runtime_policy(ScheduledBackupPolicy("\n ", "s3://b"))),
        variant(to_runtime_policy(ScheduledBackupPolicy(" ", "s3://b"))))
    checks.append({"name": CRD_SAFE_SCHEDULED_POLICY_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. the blankness test reads the same way when called directly
    exp3 = CRD_SAFE_SCHEDULED_POLICY_SECURITY_MATRIX[2][1]
    obs3 = (is_blank_schedule(""), is_blank_schedule("  "), is_blank_schedule("\t\n"),
        is_blank_schedule("0 3 * * *"), is_blank_schedule(" 0 "))
    checks.append({"name": CRD_SAFE_SCHEDULED_POLICY_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. an unparseable destination fails at admission
    exp4 = CRD_SAFE_SCHEDULED_POLICY_SECURITY_MATRIX[3][1]
    bad_dest = to_runtime_policy(ScheduledBackupPolicy("0 3 * * *", "ftp://h/x"))
    obs4 = (variant(bad_dest), bad_dest.uri,
        variant(to_runtime_policy(ScheduledBackupPolicy("0 3 * * *", ""))),
        variant(to_runtime_policy(ScheduledBackupPolicy("0 3 * * *", "s3:///p"))))
    checks.append({"name": CRD_SAFE_SCHEDULED_POLICY_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. admission reads the destination with the writers own parser
    exp5 = CRD_SAFE_SCHEDULED_POLICY_SECURITY_MATRIX[4][1]
    obs5 = (variant(to_runtime_policy(ScheduledBackupPolicy("0 3 * * *", "s3://b/p")).destination),
        to_runtime_policy(ScheduledBackupPolicy("0 3 * * *", "s3://b/p")).destination
        == parse_destination("s3://b/p"),
        to_runtime_policy(ScheduledBackupPolicy("0 3 * * *", "gs://b/p")).destination
        == parse_destination("gs://b/p"),
        to_runtime_policy(ScheduledBackupPolicy("0 3 * * *", "file:///x")).destination
        == parse_destination("file:///x"))
    checks.append({"name": CRD_SAFE_SCHEDULED_POLICY_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. the schedule is checked before the destination
    exp6 = CRD_SAFE_SCHEDULED_POLICY_SECURITY_MATRIX[5][1]
    obs6 = (variant(to_runtime_policy(ScheduledBackupPolicy("", "ftp://h"))),
        variant(to_runtime_policy(ScheduledBackupPolicy("   ", ""))),
        variant(to_runtime_policy(ScheduledBackupPolicy("0 3 * * *", "ftp://h"))))
    checks.append({"name": CRD_SAFE_SCHEDULED_POLICY_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. an admitted schedule is carried through byte for byte
    exp7 = CRD_SAFE_SCHEDULED_POLICY_SECURITY_MATRIX[6][1]
    kept = to_runtime_policy(ScheduledBackupPolicy(" 0 3 * * * ", "s3://b/p"))
    obs7 = (kept.schedule, kept.schedule == " 0 3 * * * ", len(kept.schedule))
    checks.append({"name": CRD_SAFE_SCHEDULED_POLICY_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. an admitted policy is immutable
    exp8 = CRD_SAFE_SCHEDULED_POLICY_SECURITY_MATRIX[7][1]
    obs8 = (refusal(setattr, ScheduledBackupPolicy("0 3 * * *", "s3://b"), "schedule", "x"),
        refusal(setattr, Retention(1), "max_age_seconds", 2),
        refusal(setattr, BackupPolicy("0 3 * * *", Local("/x")), "destination", Local("/y")))
    checks.append({"name": CRD_SAFE_SCHEDULED_POLICY_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. an admission refusal is returned rather than raised
    exp9 = CRD_SAFE_SCHEDULED_POLICY_SECURITY_MATRIX[8][1]
    obs9 = (refusal(to_runtime_policy, ScheduledBackupPolicy("", "s3://b")),
        refusal(to_runtime_policy, ScheduledBackupPolicy("0 3 * * *", "ftp://h")),
        variant(to_runtime_policy(ScheduledBackupPolicy("", "s3://b"))))
    checks.append({"name": CRD_SAFE_SCHEDULED_POLICY_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. the empty schedule refusal reads as a sentence
    exp10 = CRD_SAFE_SCHEDULED_POLICY_SECURITY_MATRIX[9][1]
    obs10 = (describe(EmptySchedule()),
        describe(to_runtime_policy(ScheduledBackupPolicy("  ", "s3://b"))))
    checks.append({"name": CRD_SAFE_SCHEDULED_POLICY_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. a zero retention is a configured retention not an absent one
    exp11 = CRD_SAFE_SCHEDULED_POLICY_SECURITY_MATRIX[10][1]
    zero = ScheduledBackupPolicy("0 3 * * *", "s3://b", 0)
    obs11 = (to_runtime_policy(zero).retention.max_age_seconds,
        prunes_by_age(to_runtime_policy(zero).retention),
        scheduled_policy_to_json(zero))
    checks.append({"name": CRD_SAFE_SCHEDULED_POLICY_SECURITY_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    return {
        "case_id": "crd-safe-scheduled-policy-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
