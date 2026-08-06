from __future__ import annotations

from service_backup.application.parse import parse_destination
from service_backup.application.restore import LocalObject, RemoteObject, parse_object_uri
from service_backup.infrastructure.schemes import BuildFeatures

MINIMUM_CHECKS = 11

EXACT_OBJECT_RESTORE_FETCH_BEHAVIOR_MATRIX = (
    ("a_local_object_uri_resolves_to_the_exact_path",
     ('LocalObject', '/var/lib/lumen/backup-1.json')),
    ("a_local_path_keeps_every_character_it_was_given",
     ('/a//b/', '/', 'relative/x')),
    ("an_object_store_uri_resolves_to_a_scheme_a_bucket_and_a_key",
     ('RemoteObject', 's3://', 'axiom', 'lumen/backup-1.json')),
    ("a_single_segment_key_splits_the_same_way_a_nested_one_does",
     ('axiom', 'one.json')),
    ("the_split_takes_the_first_separator_not_the_last",
     ('axiom', 'a/b/c.json')),
    ("a_google_object_uri_carries_its_own_scheme_tag",
     ('RemoteObject', 'gs://', 'axiom', 'a/b.json')),
    ("leading_separators_on_a_key_are_trimmed_and_internal_ones_survive",
     ('a//b.json', 'a/b.json/', 'k')),
    ("whitespace_around_an_object_uri_is_trimmed",
     ('k', '/x', 'axiom')),
    ("the_object_reader_and_the_destination_writer_read_the_same_text_differently",
     ('RemoteObject', 'lumen', 'S3', 'lumen', 'MissingKey', 'S3')),
    ("the_reader_accepts_the_same_three_schemes_as_the_writer",
     ('LocalObject', 'RemoteObject', 'RemoteObject', 'UnsupportedScheme')),
    ("a_resolved_object_compares_by_value_and_cannot_be_rewritten",
     (True, True, 'FrozenInstanceError')),
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


LINKED = BuildFeatures(s3=True)


def verify_exact_object_restore_fetch_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. a local object uri resolves to the exact path
    exp1 = EXACT_OBJECT_RESTORE_FETCH_BEHAVIOR_MATRIX[0][1]
    local = parse_object_uri("file:///var/lib/lumen/backup-1.json", LINKED)
    obs1 = (variant(local), local.path)
    checks.append({"name": EXACT_OBJECT_RESTORE_FETCH_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. a local path keeps every character it was given
    exp2 = EXACT_OBJECT_RESTORE_FETCH_BEHAVIOR_MATRIX[1][1]
    obs2 = (parse_object_uri("file:///a//b/", LINKED).path,
        parse_object_uri("file:///", LINKED).path,
        parse_object_uri("file://relative/x", LINKED).path)
    checks.append({"name": EXACT_OBJECT_RESTORE_FETCH_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. an object store uri resolves to a scheme a bucket and a key
    exp3 = EXACT_OBJECT_RESTORE_FETCH_BEHAVIOR_MATRIX[2][1]
    remote = parse_object_uri("s3://axiom/lumen/backup-1.json", LINKED)
    obs3 = (variant(remote), remote.scheme, remote.bucket, remote.key)
    checks.append({"name": EXACT_OBJECT_RESTORE_FETCH_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. a single segment key splits the same way a nested one does
    exp4 = EXACT_OBJECT_RESTORE_FETCH_BEHAVIOR_MATRIX[3][1]
    obs4 = (parse_object_uri("s3://axiom/one.json", LINKED).bucket,
        parse_object_uri("s3://axiom/one.json", LINKED).key)
    checks.append({"name": EXACT_OBJECT_RESTORE_FETCH_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. the split takes the first separator not the last
    exp5 = EXACT_OBJECT_RESTORE_FETCH_BEHAVIOR_MATRIX[4][1]
    nested = parse_object_uri("s3://axiom/a/b/c.json", LINKED)
    obs5 = (nested.bucket, nested.key)
    checks.append({"name": EXACT_OBJECT_RESTORE_FETCH_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. a google object uri carries its own scheme tag
    exp6 = EXACT_OBJECT_RESTORE_FETCH_BEHAVIOR_MATRIX[5][1]
    google = parse_object_uri("gs://axiom/a/b.json", LINKED)
    obs6 = (variant(google), google.scheme, google.bucket, google.key)
    checks.append({"name": EXACT_OBJECT_RESTORE_FETCH_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. leading separators on a key are trimmed and internal ones survive
    exp7 = EXACT_OBJECT_RESTORE_FETCH_BEHAVIOR_MATRIX[6][1]
    obs7 = (parse_object_uri("s3://axiom//a//b.json", LINKED).key,
        parse_object_uri("s3://axiom/a/b.json/", LINKED).key,
        parse_object_uri("gs://axiom///k", LINKED).key)
    checks.append({"name": EXACT_OBJECT_RESTORE_FETCH_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. whitespace around an object uri is trimmed
    exp8 = EXACT_OBJECT_RESTORE_FETCH_BEHAVIOR_MATRIX[7][1]
    obs8 = (parse_object_uri("  s3://axiom/k  ", LINKED).key,
        parse_object_uri(" file:///x ", LINKED).path,
        parse_object_uri("  s3://axiom/k  ", LINKED).bucket)
    checks.append({"name": EXACT_OBJECT_RESTORE_FETCH_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. the object reader and the destination writer read the same text differently
    exp9 = EXACT_OBJECT_RESTORE_FETCH_BEHAVIOR_MATRIX[8][1]
    obs9 = (variant(parse_object_uri("s3://axiom/lumen", LINKED)),
        parse_object_uri("s3://axiom/lumen", LINKED).key,
        variant(parse_destination("s3://axiom/lumen")),
        parse_destination("s3://axiom/lumen").prefix,
        variant(parse_object_uri("s3://axiom", LINKED)),
        variant(parse_destination("s3://axiom")))
    checks.append({"name": EXACT_OBJECT_RESTORE_FETCH_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. the reader accepts the same three schemes as the writer
    exp10 = EXACT_OBJECT_RESTORE_FETCH_BEHAVIOR_MATRIX[9][1]
    obs10 = (variant(parse_object_uri("file:///x", LINKED)),
        variant(parse_object_uri("s3://b/k", LINKED)),
        variant(parse_object_uri("gs://b/k", LINKED)),
        variant(parse_object_uri("ftp://b/k", LINKED)))
    checks.append({"name": EXACT_OBJECT_RESTORE_FETCH_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. a resolved object compares by value and cannot be rewritten
    exp11 = EXACT_OBJECT_RESTORE_FETCH_BEHAVIOR_MATRIX[10][1]
    obs11 = (parse_object_uri("s3://b/k", LINKED)
        == RemoteObject(scheme="s3://", bucket="b", key="k"),
        parse_object_uri("file:///x", LINKED) == LocalObject(path="/x"),
        refusal(setattr, LocalObject(path="/x"), "path", "/y"))
    checks.append({"name": EXACT_OBJECT_RESTORE_FETCH_BEHAVIOR_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    return {
        "case_id": "exact-object-restore-fetch-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
