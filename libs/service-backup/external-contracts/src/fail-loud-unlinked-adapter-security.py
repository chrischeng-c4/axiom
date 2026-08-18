from __future__ import annotations

from service_backup.application.sink import SinkKind, select_sink, sink_identity, sink_prefix
from service_backup.domain.destination import Gcs, Local, S3
from service_backup.domain.errors import describe
from service_backup.infrastructure.schemes import BuildFeatures

MINIMUM_CHECKS = 11

FAIL_LOUD_UNLINKED_ADAPTER_SECURITY_MATRIX = (
    ("a_credentials_secret_is_refused_on_both_object_store_schemes",
     ('UnsupportedCredentialSecret', 'UnsupportedCredentialSecret', 'UnsupportedCredentialSecret')),
    ("a_destination_without_a_secret_selects_a_real_sink",
     ('SinkKind', 's3', 'SinkKind', 'gcs')),
    ("the_credential_refusal_carries_the_destination_and_the_secret",
     ('s3://axiom/lumen', 'backup-creds')),
    ("the_credential_sentence_names_the_secret_and_the_alternative",
     'backup destination gs://axiom/lumen sets credentials_secret `backup-creds`, but secret-mounted credentials are not implemented; use ambient credentials or omit credentials_secret'),
    ("the_object_store_feature_is_checked_before_the_secret",
     ('SinkKind', 'unsupported-cloud', 'UnsupportedCredentialSecret')),
    ("the_google_arm_carries_no_build_feature_gate",
     ('UnsupportedCredentialSecret', 'gcs', 'gcs')),
    ("the_empty_string_is_a_present_secret",
     ('UnsupportedCredentialSecret', '', 'SinkKind')),
    ("no_cloud_destination_resolves_to_the_local_sink_by_accident",
     (False, False, False, True)),
    ("a_foreign_destination_value_is_refused_rather_than_defaulted",
     ('TypeError', 'TypeError', 'TypeError', 'TypeError')),
    ("a_credential_rejection_is_returned_rather_than_raised",
     ('accepted', 'accepted', 'UnsupportedCredentialSecret')),
    ("the_sink_identity_and_prefix_are_derived_from_the_destination_alone",
     ('local:/var/x', 'backup', 'own', 's3://b', '', 's3://b/p', 'p', 'gs://b/backup', 'backup', 'gs://b/p')),
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


def verify_fail_loud_unlinked_adapter_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. a credentials secret is refused on both object store schemes
    exp1 = FAIL_LOUD_UNLINKED_ADAPTER_SECURITY_MATRIX[0][1]
    obs1 = (variant(select_sink(S3("b", credentials_secret="sec"), LINKED)),
        variant(select_sink(Gcs("b", credentials_secret="sec"), LINKED)),
        variant(select_sink(Gcs("b", credentials_secret="sec"), UNLINKED)))
    checks.append({"name": FAIL_LOUD_UNLINKED_ADAPTER_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. a destination without a secret selects a real sink
    exp2 = FAIL_LOUD_UNLINKED_ADAPTER_SECURITY_MATRIX[1][1]
    obs2 = (variant(select_sink(S3("b"), LINKED)), select_sink(S3("b"), LINKED).value,
        variant(select_sink(Gcs("b"), LINKED)), select_sink(Gcs("b"), LINKED).value)
    checks.append({"name": FAIL_LOUD_UNLINKED_ADAPTER_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. the credential refusal carries the destination and the secret
    exp3 = FAIL_LOUD_UNLINKED_ADAPTER_SECURITY_MATRIX[2][1]
    refused = select_sink(
        S3("axiom", "lumen", credentials_secret="backup-creds"), LINKED
        )
    obs3 = (refused.destination, refused.secret)
    checks.append({"name": FAIL_LOUD_UNLINKED_ADAPTER_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. the credential sentence names the secret and the alternative
    exp4 = FAIL_LOUD_UNLINKED_ADAPTER_SECURITY_MATRIX[3][1]
    obs4 = describe(select_sink(
        Gcs("axiom", "lumen", credentials_secret="backup-creds"), LINKED))
    checks.append({"name": FAIL_LOUD_UNLINKED_ADAPTER_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. the object store feature is checked before the secret
    exp5 = FAIL_LOUD_UNLINKED_ADAPTER_SECURITY_MATRIX[4][1]
    obs5 = (variant(select_sink(S3("b", credentials_secret="sec"), UNLINKED)),
        select_sink(S3("b", credentials_secret="sec"), UNLINKED).value,
        variant(select_sink(S3("b", credentials_secret="sec"), LINKED)))
    checks.append({"name": FAIL_LOUD_UNLINKED_ADAPTER_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. the google arm carries no build feature gate
    exp6 = FAIL_LOUD_UNLINKED_ADAPTER_SECURITY_MATRIX[5][1]
    obs6 = (variant(select_sink(Gcs("b", credentials_secret="sec"), UNLINKED)),
        select_sink(Gcs("b"), UNLINKED).value, select_sink(Gcs("b"), LINKED).value)
    checks.append({"name": FAIL_LOUD_UNLINKED_ADAPTER_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. the empty string is a present secret
    exp7 = FAIL_LOUD_UNLINKED_ADAPTER_SECURITY_MATRIX[6][1]
    obs7 = (variant(select_sink(S3("b", credentials_secret=""), LINKED)),
        select_sink(S3("b", credentials_secret=""), LINKED).secret,
        variant(select_sink(S3("b", credentials_secret=None), LINKED)))
    checks.append({"name": FAIL_LOUD_UNLINKED_ADAPTER_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. no cloud destination resolves to the local sink by accident
    exp8 = FAIL_LOUD_UNLINKED_ADAPTER_SECURITY_MATRIX[7][1]
    obs8 = (select_sink(S3("b"), UNLINKED).value == SinkKind.LOCAL.value,
        select_sink(S3("b"), LINKED).value == SinkKind.LOCAL.value,
        select_sink(Gcs("b"), UNLINKED).value == SinkKind.LOCAL.value,
        select_sink(Local("/x"), UNLINKED).value == SinkKind.LOCAL.value)
    checks.append({"name": FAIL_LOUD_UNLINKED_ADAPTER_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. a foreign destination value is refused rather than defaulted
    exp9 = FAIL_LOUD_UNLINKED_ADAPTER_SECURITY_MATRIX[8][1]
    obs9 = (refusal(select_sink, "s3://b", LINKED), refusal(select_sink, None, LINKED),
        refusal(sink_prefix, "x"), refusal(sink_identity, "x"))
    checks.append({"name": FAIL_LOUD_UNLINKED_ADAPTER_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. a credential rejection is returned rather than raised
    exp10 = FAIL_LOUD_UNLINKED_ADAPTER_SECURITY_MATRIX[9][1]
    obs10 = (refusal(select_sink, S3("b", credentials_secret="s"), LINKED),
        refusal(select_sink, Gcs("b", credentials_secret="s"), UNLINKED),
        variant(select_sink(S3("b", credentials_secret="s"), LINKED)))
    checks.append({"name": FAIL_LOUD_UNLINKED_ADAPTER_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. the sink identity and prefix are derived from the destination alone
    exp11 = FAIL_LOUD_UNLINKED_ADAPTER_SECURITY_MATRIX[10][1]
    obs11 = (sink_identity(Local("/var/x")), sink_prefix(Local("/var/x")),
        sink_prefix(Local("/var/x", "own")), sink_identity(S3("b")),
        sink_prefix(S3("b")), sink_identity(S3("b", "/p/")), sink_prefix(S3("b", "/p/")),
        sink_identity(Gcs("b")), sink_prefix(Gcs("b")), sink_identity(Gcs("b", "p")))
    checks.append({"name": FAIL_LOUD_UNLINKED_ADAPTER_SECURITY_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    return {
        "case_id": "fail-loud-unlinked-adapter-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
