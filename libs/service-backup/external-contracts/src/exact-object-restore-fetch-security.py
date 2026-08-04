from __future__ import annotations

from service_backup.application.parse import parse_destination
from service_backup.application.restore import parse_object_uri, split_bucket_key
from service_backup.domain.errors import describe
from service_backup.infrastructure.schemes import BuildFeatures, scheme_names

MINIMUM_CHECKS = 13

EXACT_OBJECT_RESTORE_FETCH_SECURITY_MATRIX = (
    ("the_degenerate_object_forms_are_each_refused_by_name",
     ('MissingKey', 'MissingBucket', 'MissingKey', 'EmptyDestination')),
    ("each_refusal_carries_the_uri_or_the_scheme_it_refused",
     ('s3://axiom', 's3', 'gs', 'gs://axiom')),
    ("the_reader_is_strictly_narrower_than_the_writer_on_the_same_text",
     ('MissingKey', 'S3', 'MissingKey', 'S3', 'MissingKey', 'Gcs')),
    ("a_bucketless_object_uri_is_refused_on_both_object_store_schemes",
     ('MissingBucket', 'MissingBucket', 's3', 'gs')),
    ("a_key_that_is_only_separators_is_an_empty_key",
     ('MissingKey', 'MissingKey', 'MissingKey', 's3://axiom//')),
    ("a_local_object_uri_with_no_path_is_refused",
     ('MissingPath', 'file', 'LocalObject')),
    ("the_unsupported_refusal_carries_the_tuple_the_writer_publishes",
     (('file://', 's3://', 'gs://'), True, True, 'ftp://b/k')),
    ("the_refusal_sentences_name_what_failed",
     ('backup object URI `s3://axiom` has no object key', 's3 backup URI has no bucket', 'file backup URI has no path')),
    ("an_unlinked_build_refuses_an_object_store_read",
     ('UnlinkedAdapter', 's3://axiom/k', 's3', 'RemoteObject')),
    ("the_unlinked_refusal_is_reached_only_after_the_uri_validates",
     ('MissingKey', 'MissingBucket', 'MissingKey', 'UnlinkedAdapter')),
    ("the_google_arm_reads_in_every_build",
     ('RemoteObject', 'axiom', 'k', 'RemoteObject')),
    ("an_object_refusal_is_returned_rather_than_raised",
     ('accepted', 'accepted', 'accepted', 'accepted')),
    ("the_split_helper_refuses_the_same_three_shapes_directly",
     ('MissingKey', 'MissingBucket', 'MissingKey', ('axiom', 'k'))),
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


UNLINKED = BuildFeatures(s3=False)


def verify_exact_object_restore_fetch_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. the degenerate object forms are each refused by name
    exp1 = EXACT_OBJECT_RESTORE_FETCH_SECURITY_MATRIX[0][1]
    obs1 = (variant(parse_object_uri("s3://axiom", LINKED)),
        variant(parse_object_uri("s3:///k", LINKED)),
        variant(parse_object_uri("s3://axiom/", LINKED)),
        variant(parse_object_uri("", LINKED)))
    checks.append({"name": EXACT_OBJECT_RESTORE_FETCH_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. each refusal carries the uri or the scheme it refused
    exp2 = EXACT_OBJECT_RESTORE_FETCH_SECURITY_MATRIX[1][1]
    obs2 = (parse_object_uri("s3://axiom", LINKED).uri,
        parse_object_uri("s3:///k", LINKED).scheme,
        parse_object_uri("gs:///k", LINKED).scheme,
        parse_object_uri("gs://axiom", LINKED).uri)
    checks.append({"name": EXACT_OBJECT_RESTORE_FETCH_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. the reader is strictly narrower than the writer on the same text
    exp3 = EXACT_OBJECT_RESTORE_FETCH_SECURITY_MATRIX[2][1]
    obs3 = (variant(parse_object_uri("s3://axiom", LINKED)),
        variant(parse_destination("s3://axiom")),
        variant(parse_object_uri("s3://axiom/", LINKED)),
        variant(parse_destination("s3://axiom/")),
        variant(parse_object_uri("gs://axiom", LINKED)),
        variant(parse_destination("gs://axiom")))
    checks.append({"name": EXACT_OBJECT_RESTORE_FETCH_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. a bucketless object uri is refused on both object store schemes
    exp4 = EXACT_OBJECT_RESTORE_FETCH_SECURITY_MATRIX[3][1]
    obs4 = (variant(parse_object_uri("s3:///a/b", LINKED)),
        variant(parse_object_uri("gs:///a/b", LINKED)),
        parse_object_uri("s3:///a/b", LINKED).scheme,
        parse_object_uri("gs:///a/b", LINKED).scheme)
    checks.append({"name": EXACT_OBJECT_RESTORE_FETCH_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. a key that is only separators is an empty key
    exp5 = EXACT_OBJECT_RESTORE_FETCH_SECURITY_MATRIX[4][1]
    obs5 = (variant(parse_object_uri("s3://axiom//", LINKED)),
        variant(parse_object_uri("s3://axiom///", LINKED)),
        variant(parse_object_uri("gs://axiom//", LINKED)),
        parse_object_uri("s3://axiom//", LINKED).uri)
    checks.append({"name": EXACT_OBJECT_RESTORE_FETCH_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. a local object uri with no path is refused
    exp6 = EXACT_OBJECT_RESTORE_FETCH_SECURITY_MATRIX[5][1]
    obs6 = (variant(parse_object_uri("file://", LINKED)),
        parse_object_uri("file://", LINKED).scheme,
        variant(parse_object_uri("file:///", LINKED)))
    checks.append({"name": EXACT_OBJECT_RESTORE_FETCH_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. the unsupported refusal carries the tuple the writer publishes
    exp7 = EXACT_OBJECT_RESTORE_FETCH_SECURITY_MATRIX[6][1]
    unsupported = parse_object_uri("ftp://b/k", LINKED)
    obs7 = (unsupported.supported,
        unsupported.supported == parse_destination("ftp://b/k").supported,
        unsupported.supported == scheme_names(LINKED), unsupported.uri)
    checks.append({"name": EXACT_OBJECT_RESTORE_FETCH_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. the refusal sentences name what failed
    exp8 = EXACT_OBJECT_RESTORE_FETCH_SECURITY_MATRIX[7][1]
    obs8 = (describe(parse_object_uri("s3://axiom", LINKED)),
        describe(parse_object_uri("s3:///k", LINKED)),
        describe(parse_object_uri("file://", LINKED)))
    checks.append({"name": EXACT_OBJECT_RESTORE_FETCH_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. an unlinked build refuses an object store read
    exp9 = EXACT_OBJECT_RESTORE_FETCH_SECURITY_MATRIX[8][1]
    obs9 = (variant(parse_object_uri("s3://axiom/k", UNLINKED)),
        parse_object_uri("s3://axiom/k", UNLINKED).destination,
        parse_object_uri("s3://axiom/k", UNLINKED).feature,
        variant(parse_object_uri("s3://axiom/k", LINKED)))
    checks.append({"name": EXACT_OBJECT_RESTORE_FETCH_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. the unlinked refusal is reached only after the uri validates
    exp10 = EXACT_OBJECT_RESTORE_FETCH_SECURITY_MATRIX[9][1]
    obs10 = (variant(parse_object_uri("s3://axiom", UNLINKED)),
        variant(parse_object_uri("s3:///k", UNLINKED)),
        variant(parse_object_uri("s3://axiom/", UNLINKED)),
        variant(parse_object_uri("s3://axiom/k", UNLINKED)))
    checks.append({"name": EXACT_OBJECT_RESTORE_FETCH_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. the google arm reads in every build
    exp11 = EXACT_OBJECT_RESTORE_FETCH_SECURITY_MATRIX[10][1]
    obs11 = (variant(parse_object_uri("gs://axiom/k", UNLINKED)),
        parse_object_uri("gs://axiom/k", UNLINKED).bucket,
        parse_object_uri("gs://axiom/k", UNLINKED).key,
        variant(parse_object_uri("gs://axiom/k", LINKED)))
    checks.append({"name": EXACT_OBJECT_RESTORE_FETCH_SECURITY_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. an object refusal is returned rather than raised
    exp12 = EXACT_OBJECT_RESTORE_FETCH_SECURITY_MATRIX[11][1]
    obs12 = (refusal(parse_object_uri, "s3://axiom", LINKED),
        refusal(parse_object_uri, "", LINKED),
        refusal(parse_object_uri, "ftp://b/k", LINKED),
        refusal(split_bucket_key, "b", "s3://b", "s3"))
    checks.append({"name": EXACT_OBJECT_RESTORE_FETCH_SECURITY_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    # 13. the split helper refuses the same three shapes directly
    exp13 = EXACT_OBJECT_RESTORE_FETCH_SECURITY_MATRIX[12][1]
    obs13 = (variant(split_bucket_key("axiom", "s3://axiom", "s3")),
        variant(split_bucket_key("/k", "s3:///k", "s3")),
        variant(split_bucket_key("axiom/", "s3://axiom/", "s3")),
        split_bucket_key("axiom/k", "s3://axiom/k", "s3"))
    checks.append({"name": EXACT_OBJECT_RESTORE_FETCH_SECURITY_MATRIX[12][0], "expected": exp13,
                   "observed": obs13, "passed": obs13 == exp13})

    return {
        "case_id": "exact-object-restore-fetch-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
