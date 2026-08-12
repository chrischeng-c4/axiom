"""EC security case for #3130 -- fail-closed release-artifact verification.

Every expected value in this matrix is an EC-owned literal from #3130's
decidable R2, R6, R8, and AC3 core.  The rows give every pure entry point its
own explicit mutable, unresolved, unsigned, or wrong-identity input; no row
accepts a design-provided validity boolean as evidence of the refusal.
"""

from __future__ import annotations

from lumen.release_artifacts.admission import (
    decide_artifact_reference,
    decide_verification_request,
)
from lumen.release_artifacts.handoff import decide_handoff_content
from lumen.release_artifacts.spec import (
    ArtifactReference,
    ExpectedIdentity,
    HandoffContent,
    VerificationRequest,
)
from lumen.release_artifacts.verdict import Rejection

MINIMUM_CHECKS = 14

RELEASE_ARTIFACTS_3130_SECURITY_MATRIX = (
    ("tag_only_artifact_reference_is_rejected", "tag_only_reference"),
    ("tag_only_artifact_refusal_names_reference", "reference"),
    ("unresolved_artifact_reference_is_rejected", "unresolved_reference"),
    ("unsigned_digest_verification_request_is_rejected", "unsigned_evidence"),
    ("unsigned_request_refusal_names_signature_evidence", "signature_present"),
    ("wrong_repository_identity_is_rejected", "repository_identity_mismatch"),
    ("wrong_repository_refusal_names_expected_repository", "expected_identity.repository"),
    ("wrong_workflow_identity_is_rejected", "workflow_identity_mismatch"),
    ("wrong_workflow_refusal_names_expected_workflow", "expected_identity.workflow"),
    ("tag_only_verification_subject_is_rejected", "tag_only_subject"),
    ("unresolved_verification_subject_is_rejected", "unresolved_subject"),
    ("tag_only_handoff_proof_is_rejected", "mutable_tag_proof"),
    ("tag_only_handoff_refusal_names_declared_subjects", "declared_subjects"),
    ("unresolved_handoff_subject_is_rejected", "unresolved_subject"),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def _reference(value: str) -> ArtifactReference:
    return ArtifactReference(reference=value)


def _identity(*, repository: str = "chrischeng-c4/axiom", workflow: str = ".github/workflows/lumen-release.yml") -> ExpectedIdentity:
    return ExpectedIdentity(repository=repository, workflow=workflow)


def _request(*, subject: str, signature_present: bool = True, expected_identity: ExpectedIdentity | None = None, observed_identity: ExpectedIdentity | None = None) -> VerificationRequest:
    return VerificationRequest(
        subject=_reference(subject),
        signature_present=signature_present,
        expected_identity=expected_identity or _identity(),
        observed_identity=observed_identity or _identity(),
    )


def verify_release_artifacts_3130_security() -> dict:
    checks = []
    digest = "ghcr.io/chrischeng-c4/lumen@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    tag = "ghcr.io/chrischeng-c4/lumen:0.4.5"
    unresolved = "ghcr.io/chrischeng-c4/lumen@sha256:"

    tag_reference = decide_artifact_reference(_reference(tag))

    # 1-2. R2/R8 -- discovery tags cannot enter the artifact gate, and the
    #        typed refusal must identify the argument that was mutable.
    obs1 = _outcome(tag_reference)
    exp1 = RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[0][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = tag_reference.field_path if isinstance(tag_reference, Rejection) else "admitted"
    exp2 = RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[1][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R2/R8 -- a syntactically incomplete digest reference is not treated as
    #    equivalent to a resolved immutable subject.
    obs3 = _outcome(decide_artifact_reference(_reference(unresolved)))
    exp3 = RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[2][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    unsigned = decide_verification_request(
        _request(subject=digest, signature_present=False)
    )

    # 4-5. R8 -- explicit absence of signature evidence fails closed and names
    #        the evidence field; a default could not exercise this policy.
    obs4 = _outcome(unsigned)
    exp4 = RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[3][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    obs5 = unsigned.field_path if isinstance(unsigned, Rejection) else "admitted"
    exp5 = RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[4][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    repository_mismatch = decide_verification_request(
        _request(subject=digest, expected_identity=_identity(), observed_identity=_identity(repository="other-org/other-repo"))
    )

    # 6-7. R4/R8 -- repository identity mismatch is independently forbidden.
    obs6 = _outcome(repository_mismatch)
    exp6 = RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[5][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})
    obs7 = repository_mismatch.field_path if isinstance(repository_mismatch, Rejection) else "admitted"
    exp7 = RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[6][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    workflow_mismatch = decide_verification_request(
        _request(subject=digest, expected_identity=_identity(), observed_identity=_identity(workflow=".github/workflows/other.yml"))
    )

    # 8-9. R4/R8 -- workflow identity is a separate OIDC claim and therefore a
    #        separate fail-closed check, not an implication of repository match.
    obs8 = _outcome(workflow_mismatch)
    exp8 = RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[7][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    obs9 = workflow_mismatch.field_path if isinstance(workflow_mismatch, Rejection) else "admitted"
    exp9 = RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[8][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. R2/R8 -- verification admission repeats the tag-only refusal; checking
    #     only reference admission would let this entry point become permissive.
    obs10 = _outcome(decide_verification_request(_request(subject=tag)))
    exp10 = RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[9][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. R2/R8 -- verification admission also rejects an unresolved digest.
    obs11 = _outcome(decide_verification_request(_request(subject=unresolved)))
    exp11 = RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[10][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    tag_handoff = decide_handoff_content(
        HandoffContent(declared_subjects=(_reference(tag),), expected_identity=_identity())
    )

    # 12-13. AC3 -- handoff validation independently rejects mutable proof and
    #        tells the author that the declared proof subject is the defect.
    obs12 = _outcome(tag_handoff)
    exp12 = RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[11][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})
    obs13 = tag_handoff.field_path if isinstance(tag_handoff, Rejection) else "admitted"
    exp13 = RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[12][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    # 14. R2/AC3 -- handoff content also rejects unresolved subjects instead of
    #     accepting their spelling as a digest-bound proof.
    obs14 = _outcome(
        decide_handoff_content(
            HandoffContent(declared_subjects=(_reference(unresolved),), expected_identity=_identity())
        )
    )
    exp14 = RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[13][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    return {
        "case_id": "release-artifacts-3130-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
