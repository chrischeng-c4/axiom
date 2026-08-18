from __future__ import annotations

from service_backup.application.parse import parse_destination
from service_backup.application.sink import resolve_s3_region
from service_backup.domain.destination import Gcs, Local, S3, default_prefix, identity

MINIMUM_CHECKS = 12

DESTINATION_URI_CONTRACT_BEHAVIOR_MATRIX = (
    ("a_file_uri_resolves_to_a_local_path",
     ('Local', '/var/lib/lumen/backup', None)),
    ("an_s3_uri_splits_bucket_then_prefix",
     ('S3', 'axiom-backups', 'lumen/prod')),
    ("a_gs_uri_splits_bucket_then_prefix",
     ('Gcs', 'axiom-backups', 'lumen/prod')),
    ("each_destination_kind_renders_its_own_identity",
     ('local:/var/lib/x', 's3://b/p', 's3://b', 'gs://b/p', 'gs://b')),
    ("a_local_path_keeps_its_trailing_separator",
     ('/var/lib/lumen/', '/')),
    ("separators_around_an_s3_prefix_are_trimmed_at_the_ends_only",
     ('p', 'p', 'p//x', '')),
    ("a_gs_prefix_is_trimmed_the_same_way_an_s3_prefix_is",
     ('p', 'p', 'p//x', '')),
    ("a_bucket_only_uri_carries_the_empty_prefix",
     ('b', '', 'b', '')),
    ("the_empty_prefix_and_the_default_prefix_are_different_values",
     ('', 'backup', 'p', 'backup', 'own', 'backup')),
    ("an_identity_round_trips_a_parsed_destination",
     ('local:/var/lib/x', 's3://b/p', 's3://b', 'gs://b/p', 'gs://b')),
    ("the_parser_sets_no_transport_options_and_the_region_falls_back",
     (None, None, None, None, 'eu-west-1', 'us-east-1', 'eu-west-1')),
    ("surrounding_whitespace_on_an_operator_written_uri_is_trimmed",
     ('/var/lib/x', 'S3', 'b')),
)


def variant(value: object) -> str:
    """The name of the returned variant — the shape of an error-as-value."""
    return type(value).__name__


def verify_destination_uri_contract_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. a file uri resolves to a local path
    exp1 = DESTINATION_URI_CONTRACT_BEHAVIOR_MATRIX[0][1]
    local = parse_destination("file:///var/lib/lumen/backup")
    obs1 = (variant(local), local.path, local.prefix)
    checks.append({"name": DESTINATION_URI_CONTRACT_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. an s3 uri splits bucket then prefix
    exp2 = DESTINATION_URI_CONTRACT_BEHAVIOR_MATRIX[1][1]
    s3 = parse_destination("s3://axiom-backups/lumen/prod")
    obs2 = (variant(s3), s3.bucket, s3.prefix)
    checks.append({"name": DESTINATION_URI_CONTRACT_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. a gs uri splits bucket then prefix
    exp3 = DESTINATION_URI_CONTRACT_BEHAVIOR_MATRIX[2][1]
    gcs = parse_destination("gs://axiom-backups/lumen/prod")
    obs3 = (variant(gcs), gcs.bucket, gcs.prefix)
    checks.append({"name": DESTINATION_URI_CONTRACT_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. each destination kind renders its own identity
    exp4 = DESTINATION_URI_CONTRACT_BEHAVIOR_MATRIX[3][1]
    obs4 = (identity(Local("/var/lib/x")), identity(S3("b", "p")), identity(S3("b")),
        identity(Gcs("b", "p")), identity(Gcs("b")))
    checks.append({"name": DESTINATION_URI_CONTRACT_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. a local path keeps its trailing separator
    exp5 = DESTINATION_URI_CONTRACT_BEHAVIOR_MATRIX[4][1]
    obs5 = (parse_destination("file:///var/lib/lumen/").path,
        parse_destination("file:///").path)
    checks.append({"name": DESTINATION_URI_CONTRACT_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. separators around an s3 prefix are trimmed at the ends only
    exp6 = DESTINATION_URI_CONTRACT_BEHAVIOR_MATRIX[5][1]
    obs6 = (parse_destination("s3://b/p/").prefix,
        parse_destination("s3://b//p").prefix,
        parse_destination("s3://b/p//x").prefix,
        parse_destination("s3://b/").prefix)
    checks.append({"name": DESTINATION_URI_CONTRACT_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. a gs prefix is trimmed the same way an s3 prefix is
    exp7 = DESTINATION_URI_CONTRACT_BEHAVIOR_MATRIX[6][1]
    obs7 = (parse_destination("gs://b/p/").prefix,
        parse_destination("gs://b//p").prefix,
        parse_destination("gs://b/p//x").prefix,
        parse_destination("gs://b/").prefix)
    checks.append({"name": DESTINATION_URI_CONTRACT_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. a bucket only uri carries the empty prefix
    exp8 = DESTINATION_URI_CONTRACT_BEHAVIOR_MATRIX[7][1]
    obs8 = (parse_destination("s3://b").bucket, parse_destination("s3://b").prefix,
        parse_destination("gs://b").bucket, parse_destination("gs://b").prefix)
    checks.append({"name": DESTINATION_URI_CONTRACT_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. the empty prefix and the default prefix are different values
    exp9 = DESTINATION_URI_CONTRACT_BEHAVIOR_MATRIX[8][1]
    obs9 = (parse_destination("s3://b").prefix,
        default_prefix(parse_destination("s3://b")),
        default_prefix(S3("b", "p")), default_prefix(Local("/x")),
        default_prefix(Local("/x", "own")), default_prefix(Gcs("b")))
    checks.append({"name": DESTINATION_URI_CONTRACT_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. an identity round trips a parsed destination
    exp10 = DESTINATION_URI_CONTRACT_BEHAVIOR_MATRIX[9][1]
    obs10 = (identity(parse_destination("file:///var/lib/x")),
        identity(parse_destination("s3://b/p")),
        identity(parse_destination("s3://b")),
        identity(parse_destination("gs://b/p")),
        identity(parse_destination("gs://b")))
    checks.append({"name": DESTINATION_URI_CONTRACT_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. the parser sets no transport options and the region falls back
    exp11 = DESTINATION_URI_CONTRACT_BEHAVIOR_MATRIX[10][1]
    parsed = parse_destination("s3://b/p")
    obs11 = (parsed.region, parsed.endpoint, parsed.credentials_secret,
        resolve_s3_region(S3("b")), resolve_s3_region(S3("b", region="eu-west-1")),
        resolve_s3_region(S3("b", endpoint="http://minio:9000")),
        resolve_s3_region(S3("b", region="eu-west-1", endpoint="http://minio:9000")))
    checks.append({"name": DESTINATION_URI_CONTRACT_BEHAVIOR_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. surrounding whitespace on an operator written uri is trimmed
    exp12 = DESTINATION_URI_CONTRACT_BEHAVIOR_MATRIX[11][1]
    obs12 = (parse_destination("  file:///var/lib/x  ").path,
        variant(parse_destination(" s3://b/p ")),
        parse_destination(" s3://b/p ").bucket)
    checks.append({"name": DESTINATION_URI_CONTRACT_BEHAVIOR_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    return {
        "case_id": "destination-uri-contract-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
