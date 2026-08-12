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

MINIMUM_CHECKS = 17

RELEASE_ARTIFACTS_3130_SECURITY_MATRIX = (
    ("tag_only_artifact_reference_is_rejected", "tag_only_reference"),
    ("tag_only_artifact_refusal_names_reference", "reference"),
    ("unresolved_artifact_reference_is_rejected", "unresolved_reference"),
    ("foreign_repository_digest_reference_is_rejected", "foreign_repository_reference"),
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
    ("handoff_missing_repository_identity_is_rejected", "missing_repository_identity"),
    ("handoff_missing_workflow_identity_is_rejected", "missing_workflow_identity"),
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
    foreign_digest = "ghcr.io/other/lumen@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"

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

    # 4. R2 -- syntactically valid immutable references from another image
    #    path cannot be substituted for Lumen's canonical GHCR subject.
    obs4 = _outcome(decide_artifact_reference(_reference(foreign_digest)))
    exp4 = RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[3][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5-6. R8 -- explicit absence of signature evidence fails closed and names
    #        the evidence field; a default could not exercise this policy.
    obs5 = _outcome(unsigned)
    exp5 = RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[4][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    obs6 = unsigned.field_path if isinstance(unsigned, Rejection) else "admitted"
    exp6 = RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[5][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    repository_mismatch = decide_verification_request(
        _request(subject=digest, expected_identity=_identity(), observed_identity=_identity(repository="other-org/other-repo"))
    )

    # 7-8. R4/R8 -- repository identity mismatch is independently forbidden.
    obs7 = _outcome(repository_mismatch)
    exp7 = RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[6][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    obs8 = repository_mismatch.field_path if isinstance(repository_mismatch, Rejection) else "admitted"
    exp8 = RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[7][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    workflow_mismatch = decide_verification_request(
        _request(subject=digest, expected_identity=_identity(), observed_identity=_identity(workflow=".github/workflows/other.yml"))
    )

    # 9-10. R4/R8 -- workflow identity is a separate OIDC claim and therefore a
    #        separate fail-closed check, not an implication of repository match.
    obs9 = _outcome(workflow_mismatch)
    exp9 = RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[8][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})
    obs10 = workflow_mismatch.field_path if isinstance(workflow_mismatch, Rejection) else "admitted"
    exp10 = RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[9][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. R2/R8 -- verification admission repeats the tag-only refusal; checking
    #     only reference admission would let this entry point become permissive.
    obs11 = _outcome(decide_verification_request(_request(subject=tag)))
    exp11 = RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[10][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12. R2/R8 -- verification admission also rejects an unresolved digest.
    obs12 = _outcome(decide_verification_request(_request(subject=unresolved)))
    exp12 = RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[11][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    tag_handoff = decide_handoff_content(
        HandoffContent(declared_subjects=(_reference(tag),), expected_identity=_identity())
    )

    # 13-14. AC3 -- handoff validation independently rejects mutable proof and
    #        tells the author that the declared proof subject is the defect.
    obs13 = _outcome(tag_handoff)
    exp13 = RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[12][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})
    obs14 = tag_handoff.field_path if isinstance(tag_handoff, Rejection) else "admitted"
    exp14 = RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[13][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    # 15. R2/AC3 -- handoff content also rejects unresolved subjects instead of
    #     accepting their spelling as a digest-bound proof.
    obs15 = _outcome(
        decide_handoff_content(
            HandoffContent(declared_subjects=(_reference(unresolved),), expected_identity=_identity())
        )
    )
    exp15 = RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[14][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    # 16-17. AC3 -- handoff proof needs both independently useful identity
    #        fields; an immutable subject alone is never sufficient guidance.
    missing_repository = decide_handoff_content(
        HandoffContent(declared_subjects=(_reference(digest),), expected_identity=_identity(repository=""))
    )
    obs16 = _outcome(missing_repository)
    exp16 = RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[15][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})

    missing_workflow = decide_handoff_content(
        HandoffContent(declared_subjects=(_reference(digest),), expected_identity=_identity(workflow=""))
    )
    obs17 = _outcome(missing_workflow)
    exp17 = RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[16][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_SECURITY_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})

    return {
        "case_id": "release-artifacts-3130-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
