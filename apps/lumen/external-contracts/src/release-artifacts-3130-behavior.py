"""EC behavior case for #3130 -- digest-bound Lumen release artifacts.

Every expected value in this matrix is an EC-owned literal transcribed from
#3130's decidable R2, R6, R8, and AC3 core.  This case deliberately drives the
pure release-artifact model only: it establishes which references and handoff
plans the design admits, without claiming that GHCR, GitHub Actions, cosign, or
an attestation service was contacted.
"""

from __future__ import annotations

from lumen.release_artifacts.admission import (
    decide_artifact_reference,
    decide_verification_request,
)
from lumen.release_artifacts.handoff import (
    decide_handoff_content,
    render_verification_plan,
)
from lumen.release_artifacts.spec import (
    ArtifactReference,
    ExpectedIdentity,
    HandoffContent,
    VerificationRequest,
)
from lumen.release_artifacts.verdict import Rejection

MINIMUM_CHECKS = 9

RELEASE_ARTIFACTS_3130_BEHAVIOR_MATRIX = (
    ("canonical_ghcr_digest_reference_is_admitted", "admitted"),
    ("admitted_reference_preserves_its_immutable_subject", "ghcr.io/chrischeng-c4/lumen@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
    ("signed_matching_digest_verification_request_is_admitted", "admitted"),
    ("verification_plan_subject_is_the_admitted_digest", "ghcr.io/chrischeng-c4/lumen@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
    ("verification_plan_names_the_expected_repository", "chrischeng-c4/axiom"),
    ("verification_plan_names_the_expected_release_workflow", ".github/workflows/lumen-release.yml"),
    ("verification_plan_contains_digest_pinned_cosign_and_github_commands", ("cosign", "github-attestation", "github-sbom")),
    ("every_rendered_verification_command_uses_the_admitted_digest", (
        "ghcr.io/chrischeng-c4/lumen@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "ghcr.io/chrischeng-c4/lumen@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "ghcr.io/chrischeng-c4/lumen@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    )),
    ("digest_bound_handoff_with_required_identity_is_admitted", "admitted"),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def _reference(value: str) -> ArtifactReference:
    return ArtifactReference(reference=value)


def _identity() -> ExpectedIdentity:
    return ExpectedIdentity(
        repository="chrischeng-c4/axiom",
        workflow=".github/workflows/lumen-release.yml",
    )


def verify_release_artifacts_3130_behavior() -> dict:
    checks = []
    digest = "ghcr.io/chrischeng-c4/lumen@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    identity = _identity()
    admitted_reference = decide_artifact_reference(_reference(digest))

    # 1. R2 -- the sole admissible release subject is the stated GHCR image at
    #    an immutable digest; a version tag is only discovery, never proof.
    obs1 = _outcome(admitted_reference)
    exp1 = RELEASE_ARTIFACTS_3130_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R2 -- admission carries the caller-supplied immutable subject rather
    #    than silently reconstructing it from a mutable version tag.
    obs2 = admitted_reference.reference if not isinstance(admitted_reference, Rejection) else "refused"
    exp2 = RELEASE_ARTIFACTS_3130_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    request = VerificationRequest(
        subject=_reference(digest),
        signature_present=True,
        expected_identity=identity,
        observed_identity=identity,
    )
    admitted_request = decide_verification_request(request)

    # 3. R8 -- a digest with explicitly supplied signature evidence and the
    #    matching repository/workflow identity remains an admissible request.
    obs3 = _outcome(admitted_request)
    exp3 = RELEASE_ARTIFACTS_3130_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    plan = render_verification_plan(admitted_reference, identity)

    # 4. R2/R6 -- every generated verification command is bound to the digest.
    obs4 = plan.subject
    exp4 = RELEASE_ARTIFACTS_3130_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R6 -- the plan carries the repository identity supplied by the caller.
    obs5 = plan.expected_repository
    exp5 = RELEASE_ARTIFACTS_3130_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R6 -- it independently carries the intended release-workflow identity.
    obs6 = plan.expected_workflow
    exp6 = RELEASE_ARTIFACTS_3130_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R6 -- the fixed plan supplies copyable cosign, provenance, and SBOM
    #    verification command kinds, all using the digest subject above.
    obs7 = tuple(command.kind for command in plan.commands)
    exp7 = RELEASE_ARTIFACTS_3130_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    handoff = decide_handoff_content(
        HandoffContent(
            declared_subjects=(_reference(digest),),
            expected_identity=identity,
        )
    )

    # 8. R6/AC3 -- the plan subject alone cannot make its rendered commands
    #    digest-bound: each copyable command must carry the admitted subject.
    obs8 = tuple(command.subject for command in plan.commands)
    exp8 = RELEASE_ARTIFACTS_3130_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. AC3 -- a nearby fully specified digest handoff is admitted, proving
    #    that fail-closed validation is not a blanket rejection of handoffs.
    obs9 = _outcome(handoff)
    exp9 = RELEASE_ARTIFACTS_3130_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": RELEASE_ARTIFACTS_3130_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    return {
        "case_id": "release-artifacts-3130-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
