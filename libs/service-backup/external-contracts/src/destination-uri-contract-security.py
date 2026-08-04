from __future__ import annotations

from service_backup.application.parse import parse_destination
from service_backup.application.restore import parse_object_uri
from service_backup.domain.destination import default_prefix, identity
from service_backup.domain.errors import EmptySchedule, describe
from service_backup.infrastructure.schemes import BuildFeatures, scheme_names

MINIMUM_CHECKS = 13

DESTINATION_URI_CONTRACT_SECURITY_MATRIX = (
    ("an_empty_bucket_is_refused_for_both_object_store_schemes",
     ('MissingBucket', 'MissingBucket', 's3', 'gs')),
    ("a_scheme_with_nothing_after_it_is_refused",
     ('MissingBucket', 's3', 'MissingBucket', 'gs', 'MissingPath', 'file')),
    ("an_empty_destination_is_its_own_refusal",
     ('EmptyDestination', 'EmptyDestination', 'EmptyDestination')),
    ("the_unsupported_refusal_carries_the_uri_and_every_accepted_scheme",
     ('UnsupportedScheme', 'ftp://host/x', ('file://', 's3://', 'gs://'))),
    ("the_carried_scheme_tuple_matches_the_published_inventory",
     (True, True, ('file://', 's3://', 'gs://'))),
    ("each_shape_refusal_reads_as_a_sentence_naming_what_failed",
     ('backup destination URI is empty', 'file backup URI has no path', 's3 backup URI has no bucket', 'backup object URI `s3://b` has no object key')),
    ("the_unsupported_sentence_lists_the_alternatives",
     'unsupported backup destination URI `ftp://h/x`; use file://, s3://, gs://'),
    ("a_shape_refusal_is_returned_rather_than_raised",
     ('accepted', 'accepted', 'accepted', 'UnsupportedScheme', 'EmptyDestination', 'MissingBucket')),
    ("a_uri_that_is_only_separators_has_no_bucket",
     ('MissingBucket', 'MissingBucket', '', 'b')),
    ("a_google_uri_that_fails_to_split_returns_the_error_not_a_destination",
     ('MissingBucket', 'MissingBucket', 'Gcs')),
    ("the_scheme_prefix_must_match_exactly_and_in_lower_case",
     ('UnsupportedScheme', 'UnsupportedScheme', 'UnsupportedScheme', 'UnsupportedScheme')),
    ("a_foreign_destination_value_is_refused_rather_than_defaulted",
     ('TypeError', 'TypeError', 'TypeError', 'TypeError')),
    ("an_unknown_error_variant_has_no_sentence",
     ('TypeError', 'TypeError', 'backup schedule must not be empty')),
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


def verify_destination_uri_contract_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. an empty bucket is refused for both object store schemes
    exp1 = DESTINATION_URI_CONTRACT_SECURITY_MATRIX[0][1]
    obs1 = (variant(parse_destination("s3:///p")), variant(parse_destination("gs:///p")),
        parse_destination("s3:///p").scheme, parse_destination("gs:///p").scheme)
    checks.append({"name": DESTINATION_URI_CONTRACT_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. a scheme with nothing after it is refused
    exp2 = DESTINATION_URI_CONTRACT_SECURITY_MATRIX[1][1]
    obs2 = (variant(parse_destination("s3://")), parse_destination("s3://").scheme,
        variant(parse_destination("gs://")), parse_destination("gs://").scheme,
        variant(parse_destination("file://")), parse_destination("file://").scheme)
    checks.append({"name": DESTINATION_URI_CONTRACT_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. an empty destination is its own refusal
    exp3 = DESTINATION_URI_CONTRACT_SECURITY_MATRIX[2][1]
    obs3 = (variant(parse_destination("")), variant(parse_destination("   ")),
        variant(parse_destination("\t\n")))
    checks.append({"name": DESTINATION_URI_CONTRACT_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. the unsupported refusal carries the uri and every accepted scheme
    exp4 = DESTINATION_URI_CONTRACT_SECURITY_MATRIX[3][1]
    unsupported = parse_destination("ftp://host/x")
    obs4 = (variant(unsupported), unsupported.uri, unsupported.supported)
    checks.append({"name": DESTINATION_URI_CONTRACT_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. the carried scheme tuple matches the published inventory
    exp5 = DESTINATION_URI_CONTRACT_SECURITY_MATRIX[4][1]
    obs5 = (parse_destination("ftp://h").supported == scheme_names(LINKED),
        parse_destination("ftp://h").supported == scheme_names(UNLINKED),
        scheme_names(BuildFeatures()))
    checks.append({"name": DESTINATION_URI_CONTRACT_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. each shape refusal reads as a sentence naming what failed
    exp6 = DESTINATION_URI_CONTRACT_SECURITY_MATRIX[5][1]
    obs6 = (describe(parse_destination("")), describe(parse_destination("file://")),
        describe(parse_destination("s3://")),
        describe(parse_object_uri("s3://b", LINKED)))
    checks.append({"name": DESTINATION_URI_CONTRACT_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. the unsupported sentence lists the alternatives
    exp7 = DESTINATION_URI_CONTRACT_SECURITY_MATRIX[6][1]
    obs7 = describe(parse_destination("ftp://h/x"))
    checks.append({"name": DESTINATION_URI_CONTRACT_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. a shape refusal is returned rather than raised
    exp8 = DESTINATION_URI_CONTRACT_SECURITY_MATRIX[7][1]
    obs8 = (refusal(parse_destination, ""), refusal(parse_destination, "ftp://h"),
        refusal(parse_destination, "s3://"), variant(parse_destination("ftp://h")),
        variant(parse_destination("")), variant(parse_destination("s3://")))
    checks.append({"name": DESTINATION_URI_CONTRACT_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. a uri that is only separators has no bucket
    exp9 = DESTINATION_URI_CONTRACT_SECURITY_MATRIX[8][1]
    obs9 = (variant(parse_destination("s3:////")), variant(parse_destination("gs:////")),
        parse_destination("s3://b///").prefix, parse_destination("s3://b///").bucket)
    checks.append({"name": DESTINATION_URI_CONTRACT_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. a google uri that fails to split returns the error not a destination
    exp10 = DESTINATION_URI_CONTRACT_SECURITY_MATRIX[9][1]
    obs10 = (variant(parse_destination("gs:///p")), variant(parse_destination("gs://")),
        variant(parse_destination("gs://b/p")))
    checks.append({"name": DESTINATION_URI_CONTRACT_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. the scheme prefix must match exactly and in lower case
    exp11 = DESTINATION_URI_CONTRACT_SECURITY_MATRIX[10][1]
    obs11 = (variant(parse_destination("s3:/b/p")), variant(parse_destination("file:/x")),
        variant(parse_destination("S3://b")), variant(parse_destination("gs:/b")))
    checks.append({"name": DESTINATION_URI_CONTRACT_SECURITY_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. a foreign destination value is refused rather than defaulted
    exp12 = DESTINATION_URI_CONTRACT_SECURITY_MATRIX[11][1]
    obs12 = (refusal(identity, "s3://b"), refusal(default_prefix, "s3://b"),
        refusal(identity, None), refusal(default_prefix, None))
    checks.append({"name": DESTINATION_URI_CONTRACT_SECURITY_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    # 13. an unknown error variant has no sentence
    exp13 = DESTINATION_URI_CONTRACT_SECURITY_MATRIX[12][1]
    obs13 = (refusal(describe, "boom"), refusal(describe, None),
        describe(EmptySchedule()))
    checks.append({"name": DESTINATION_URI_CONTRACT_SECURITY_MATRIX[12][0], "expected": exp13,
                   "observed": obs13, "passed": obs13 == exp13})

    return {
        "case_id": "destination-uri-contract-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
