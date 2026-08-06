from __future__ import annotations

from service_backup.application.runner import plan_prune
from service_backup.domain.policy import Retention, is_expired
from service_backup.infrastructure.keys import build_key, list_prefix, parse_backup_key

MINIMUM_CHECKS = 13

RETAINED_OBJECT_WRITE_SECURITY_MATRIX = (
    ("the_inverse_parser_recovers_the_timestamp_the_writer_stamped",
     (1767225600, 1767225600, 1767225600)),
    ("this_prefix_with_a_foreign_body_is_not_this_crates_key",
     (None, None, None, None)),
    ("a_signed_separated_or_non_ascii_timestamp_body_is_not_decimal",
     (None, None, None, None, None)),
    ("an_empty_timestamp_body_is_refused_before_conversion",
     (None, None, 0)),
    ("a_sibling_prefix_that_starts_with_the_real_one_is_not_a_match",
     (None, None, 1, None)),
    ("a_key_nested_one_level_deeper_belongs_to_the_deeper_prefix",
     (None, 1)),
    ("a_prune_plan_selects_exactly_the_expired_keys",
     (('lumen/backup-1000.json',), ('lumen/backup-1000.json', 'lumen/backup-2000.json'), 2)),
    ("a_foreign_object_sharing_the_prefix_is_never_selected",
     ('lumen/backup-1000.json',)),
    ("an_empty_prefix_selects_only_bare_keys",
     ('backup-1000.json',)),
    ("the_expiry_boundary_is_exclusive_on_the_cutoff_second",
     (False, False, True, False)),
    ("retention_measures_age_backward_from_now",
     (True, False, True, False)),
    ("an_unparseable_listing_entry_is_skipped_rather_than_raised",
     ('accepted', 'accepted', None)),
    ("the_written_key_and_the_listing_prefix_agree_on_the_boundary",
     (True, 'lumen/', 'lumen/backup-1.json', None, 'backup-1.json')),
)


def refusal(fn, *args, **kwargs) -> str:
    """The exception type a refusing call raises, or 'accepted' if it returns."""
    try:
        fn(*args, **kwargs)
    except Exception as exc:  # noqa: BLE001 — the type is the observation
        return type(exc).__name__
    return "accepted"


TS = 1767225600


def verify_retained_object_write_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. the inverse parser recovers the timestamp the writer stamped
    exp1 = RETAINED_OBJECT_WRITE_SECURITY_MATRIX[0][1]
    obs1 = (parse_backup_key("lumen", build_key("lumen", TS)),
        parse_backup_key("", build_key("", TS)),
        parse_backup_key("a/b", build_key("a/b", TS)))
    checks.append({"name": RETAINED_OBJECT_WRITE_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. this prefix with a foreign body is not this crates key
    exp2 = RETAINED_OBJECT_WRITE_SECURITY_MATRIX[1][1]
    obs2 = (parse_backup_key("lumen", "lumen/backup-abc.json"),
        parse_backup_key("lumen", "lumen/backup-.json"),
        parse_backup_key("lumen", "lumen/report-1.json"),
        parse_backup_key("lumen", "lumen/backup-1767225600.txt"))
    checks.append({"name": RETAINED_OBJECT_WRITE_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. a signed separated or non ascii timestamp body is not decimal
    exp3 = RETAINED_OBJECT_WRITE_SECURITY_MATRIX[2][1]
    obs3 = (parse_backup_key("lumen", "lumen/backup-+1.json"),
        parse_backup_key("lumen", "lumen/backup--1.json"),
        parse_backup_key("lumen", "lumen/backup-1_0.json"),
        parse_backup_key("lumen", "lumen/backup-１.json"),
        parse_backup_key("lumen", "lumen/backup- 1.json"))
    checks.append({"name": RETAINED_OBJECT_WRITE_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. an empty timestamp body is refused before conversion
    exp4 = RETAINED_OBJECT_WRITE_SECURITY_MATRIX[3][1]
    obs4 = (parse_backup_key("lumen", "lumen/backup-.json"),
        parse_backup_key("", "backup-.json"), parse_backup_key("", "backup-0.json"))
    checks.append({"name": RETAINED_OBJECT_WRITE_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. a sibling prefix that starts with the real one is not a match
    exp5 = RETAINED_OBJECT_WRITE_SECURITY_MATRIX[4][1]
    obs5 = (parse_backup_key("lumen", "lumen-old/backup-1.json"),
        parse_backup_key("lumen", "lumenx/backup-1.json"),
        parse_backup_key("lumen", "lumen/backup-1.json"),
        parse_backup_key("lumen", "lumenbackup-1.json"))
    checks.append({"name": RETAINED_OBJECT_WRITE_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. a key nested one level deeper belongs to the deeper prefix
    exp6 = RETAINED_OBJECT_WRITE_SECURITY_MATRIX[5][1]
    obs6 = (parse_backup_key("lumen", "lumen/sub/backup-1.json"),
        parse_backup_key("lumen/sub", "lumen/sub/backup-1.json"))
    checks.append({"name": RETAINED_OBJECT_WRITE_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. a prune plan selects exactly the expired keys
    exp7 = RETAINED_OBJECT_WRITE_SECURITY_MATRIX[6][1]
    listing = ("lumen/backup-1000.json", "lumen/backup-2000.json",
        "lumen/backup-3000.json", "lumen/foreign.json",
        "other/backup-1000.json")
    obs7 = (plan_prune(listing, "lumen", 3000, Retention(1500)),
        plan_prune(listing, "lumen", 3000, Retention(500)),
        len(plan_prune(listing, "lumen", 3000, Retention(0))))
    checks.append({"name": RETAINED_OBJECT_WRITE_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. a foreign object sharing the prefix is never selected
    exp8 = RETAINED_OBJECT_WRITE_SECURITY_MATRIX[7][1]
    obs8 = plan_prune(("lumen/backup-1000.json", "lumen/notes.json",
        "lumen/backup-x.json", "lumen-old/backup-1000.json"),
        "lumen", 10**9, Retention(0))
    checks.append({"name": RETAINED_OBJECT_WRITE_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. an empty prefix selects only bare keys
    exp9 = RETAINED_OBJECT_WRITE_SECURITY_MATRIX[8][1]
    obs9 = plan_prune(("backup-1000.json", "lumen/backup-1000.json", "backup-x.json"),
        "", 10**9, Retention(0))
    checks.append({"name": RETAINED_OBJECT_WRITE_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. the expiry boundary is exclusive on the cutoff second
    exp10 = RETAINED_OBJECT_WRITE_SECURITY_MATRIX[9][1]
    obs10 = (is_expired(999, 1000, Retention(1)), is_expired(1000, 1000, Retention(1)),
        is_expired(998, 1000, Retention(1)), is_expired(0, 1000, Retention(None)))
    checks.append({"name": RETAINED_OBJECT_WRITE_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. retention measures age backward from now
    exp11 = RETAINED_OBJECT_WRITE_SECURITY_MATRIX[10][1]
    obs11 = (is_expired(0, 100, Retention(50)), is_expired(60, 100, Retention(50)),
        is_expired(49, 100, Retention(50)), is_expired(50, 100, Retention(50)))
    checks.append({"name": RETAINED_OBJECT_WRITE_SECURITY_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. an unparseable listing entry is skipped rather than raised
    exp12 = RETAINED_OBJECT_WRITE_SECURITY_MATRIX[11][1]
    obs12 = (refusal(parse_backup_key, "lumen", ""),
        refusal(plan_prune, ("",), "lumen", 1, Retention(0)),
        parse_backup_key("lumen", ""))
    checks.append({"name": RETAINED_OBJECT_WRITE_SECURITY_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    # 13. the written key and the listing prefix agree on the boundary
    exp13 = RETAINED_OBJECT_WRITE_SECURITY_MATRIX[12][1]
    obs13 = (build_key("lumen", 1).startswith(list_prefix("lumen")), list_prefix("lumen"),
        build_key("lumen", 1), list_prefix(""), build_key("", 1))
    checks.append({"name": RETAINED_OBJECT_WRITE_SECURITY_MATRIX[12][0], "expected": exp13,
                   "observed": obs13, "passed": obs13 == exp13})

    return {
        "case_id": "retained-object-write-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
