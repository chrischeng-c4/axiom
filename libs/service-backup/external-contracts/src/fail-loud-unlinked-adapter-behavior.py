from __future__ import annotations

from service_backup.application.parse import parse_destination
from service_backup.application.sink import SinkKind, select_sink, unlinked_error
from service_backup.domain.destination import Gcs, Local, S3
from service_backup.domain.errors import describe
from service_backup.infrastructure.schemes import BuildFeatures, find_scheme, scheme_names, supported_schemes, unavailable_schemes

MINIMUM_CHECKS = 10

FAIL_LOUD_UNLINKED_ADAPTER_BEHAVIOR_MATRIX = (
    ("an_object_store_uri_parses_identically_in_both_builds",
     ('S3', True, ('file://', 's3://', 'gs://'))),
    ("the_selected_sink_differs_between_the_two_builds",
     ('s3', 'unsupported-cloud')),
    ("a_local_or_google_destination_is_unaffected_by_the_object_store_feature",
     ('local', 'local', 'gcs', 'gcs')),
    ("every_sink_kind_has_a_stable_wire_name",
     ('local', 's3', 'gcs', 'unsupported-cloud')),
    ("the_unlinked_refusal_names_the_destination_and_the_feature",
     ('UnlinkedAdapter', 's3://axiom/lumen', 's3')),
    ("the_unlinked_sentence_names_the_remedy",
     'backup destination s3://axiom/lumen needs the `s3` feature; rebuild with --features s3 or use a local destination'),
    ("the_unlinked_refusal_uses_the_destinations_own_identity",
     ('local:/var/x', 'gs://b/p', 's3://b')),
    ("sink_availability_is_published_per_scheme_and_per_build",
     ((('file://', True), ('s3://', True), ('gs://', True)), (('file://', True), ('s3://', False), ('gs://', True)))),
    ("the_unavailable_list_names_exactly_the_unlinked_schemes",
     ((), ('s3://',), False, True, None)),
    ("the_default_build_has_no_object_store_linked",
     (False, True, ('s3://',))),
)


def variant(value: object) -> str:
    """The name of the returned variant — the shape of an error-as-value."""
    return type(value).__name__


LINKED = BuildFeatures(s3=True)


UNLINKED = BuildFeatures(s3=False)


def verify_fail_loud_unlinked_adapter_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. an object store uri parses identically in both builds
    exp1 = FAIL_LOUD_UNLINKED_ADAPTER_BEHAVIOR_MATRIX[0][1]
    obs1 = (variant(parse_destination("s3://b/p")),
        scheme_names(LINKED) == scheme_names(UNLINKED), scheme_names(LINKED))
    checks.append({"name": FAIL_LOUD_UNLINKED_ADAPTER_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. the selected sink differs between the two builds
    exp2 = FAIL_LOUD_UNLINKED_ADAPTER_BEHAVIOR_MATRIX[1][1]
    obs2 = (select_sink(S3("b", "p"), LINKED).value,
        select_sink(S3("b", "p"), UNLINKED).value)
    checks.append({"name": FAIL_LOUD_UNLINKED_ADAPTER_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. a local or google destination is unaffected by the object store feature
    exp3 = FAIL_LOUD_UNLINKED_ADAPTER_BEHAVIOR_MATRIX[2][1]
    obs3 = (select_sink(Local("/x"), LINKED).value, select_sink(Local("/x"), UNLINKED).value,
        select_sink(Gcs("b"), LINKED).value, select_sink(Gcs("b"), UNLINKED).value)
    checks.append({"name": FAIL_LOUD_UNLINKED_ADAPTER_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. every sink kind has a stable wire name
    exp4 = FAIL_LOUD_UNLINKED_ADAPTER_BEHAVIOR_MATRIX[3][1]
    obs4 = (SinkKind.LOCAL.value, SinkKind.S3.value, SinkKind.GCS.value,
        SinkKind.UNSUPPORTED_CLOUD.value)
    checks.append({"name": FAIL_LOUD_UNLINKED_ADAPTER_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. the unlinked refusal names the destination and the feature
    exp5 = FAIL_LOUD_UNLINKED_ADAPTER_BEHAVIOR_MATRIX[4][1]
    unlinked = unlinked_error(S3("axiom", "lumen"))
    obs5 = (variant(unlinked), unlinked.destination, unlinked.feature)
    checks.append({"name": FAIL_LOUD_UNLINKED_ADAPTER_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. the unlinked sentence names the remedy
    exp6 = FAIL_LOUD_UNLINKED_ADAPTER_BEHAVIOR_MATRIX[5][1]
    obs6 = describe(unlinked_error(S3("axiom", "lumen")))
    checks.append({"name": FAIL_LOUD_UNLINKED_ADAPTER_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. the unlinked refusal uses the destinations own identity
    exp7 = FAIL_LOUD_UNLINKED_ADAPTER_BEHAVIOR_MATRIX[6][1]
    obs7 = (unlinked_error(Local("/var/x")).destination,
        unlinked_error(Gcs("b", "p")).destination,
        unlinked_error(S3("b")).destination)
    checks.append({"name": FAIL_LOUD_UNLINKED_ADAPTER_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. sink availability is published per scheme and per build
    exp8 = FAIL_LOUD_UNLINKED_ADAPTER_BEHAVIOR_MATRIX[7][1]
    obs8 = (tuple((s.scheme, s.sink_available) for s in supported_schemes(LINKED)),
        tuple((s.scheme, s.sink_available) for s in supported_schemes(UNLINKED)))
    checks.append({"name": FAIL_LOUD_UNLINKED_ADAPTER_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. the unavailable list names exactly the unlinked schemes
    exp9 = FAIL_LOUD_UNLINKED_ADAPTER_BEHAVIOR_MATRIX[8][1]
    obs9 = (unavailable_schemes(LINKED), unavailable_schemes(UNLINKED),
        find_scheme("s3://", UNLINKED).sink_available,
        find_scheme("s3://", LINKED).sink_available, find_scheme("ftp://", LINKED))
    checks.append({"name": FAIL_LOUD_UNLINKED_ADAPTER_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. the default build has no object store linked
    exp10 = FAIL_LOUD_UNLINKED_ADAPTER_BEHAVIOR_MATRIX[9][1]
    obs10 = (BuildFeatures().s3, BuildFeatures(s3=True).s3,
        unavailable_schemes(BuildFeatures()))
    checks.append({"name": FAIL_LOUD_UNLINKED_ADAPTER_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    return {
        "case_id": "fail-loud-unlinked-adapter-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
