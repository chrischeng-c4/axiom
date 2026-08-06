"Tech design for WI #2879: [12/18] lumen auth phase 2: prove two-hop KSA/RBAC authorization on GKE.\n\n@spec #2879"

from __future__ import annotations


__aw_artifact_id__ = "artifact:security-access-kubernetes-native-deployment/work-item-12-18-lumen-auth-phase-2-prove-two-hop-ksa-rbac-authorization-on-wi-2879"
__aw_work_item__ = "2879"
__aw_native_handwrite_targets__ = (
    "acceptance/gcp/scripts/verify-lumen-auth.sh",
)


def design_contract() -> str:
    """Express the executable design contract for this bounded change."""

    return "lumen_auth_redaction_audit_and_destroy"


def redaction_audit_contract() -> dict[str, str]:
    """Return the frozen shell boundary that later materialization must preserve."""

    return {
        "target": "acceptance/gcp/scripts/verify-lumen-auth.sh",
        "neither": "skip callback and preserve ordinary direct GKE execution",
        "auditor_only": "fail",
        "audit_path_only": "fail",
        "both": "invoke lumen_auth_redaction_audit_and_destroy",
        "required_expansions": "${LUMEN_AUTH_REDACTION_AUDITOR:?required} ${LUMEN_AUTH_REDACTION_AUDIT_PATH:?required}",
        "order": "write lumen-auth-acceptance.json -> auditor -> rm -rf $SECRET_DIR -> SECRET_DIR=\"\" -> final success echo",
        "verification": "bash -n acceptance/gcp/scripts/verify-lumen-auth.sh; python3 -m unittest discover -s apps/lumen/external-contracts/tests/unit",
        "ec_verify": "aw ec verify --project lumen --stage cb --wi 2879",
    }


class LumenAuthRedactionAuditAndDestroy:
    """Bind the retained acceptance evidence audit to immediate secret cleanup."""

    pass


def run_redaction_audit_then_destroy_secret_dir() -> None:
    """Require both EC variables, audit evidence plus secrets, then remove secrets.

    The later shell implementation runs only when both
    ``LUMEN_AUTH_REDACTION_AUDITOR`` and
    ``LUMEN_AUTH_REDACTION_AUDIT_PATH`` are set.  A one-sided configuration
    fails; the callback receives ``$EVIDENCE_DIR`` and ``$SECRET_DIR``, writes
    its result to the required audit path, then immediately removes
    ``$SECRET_DIR`` and clears ``SECRET_DIR`` without intervening output.
    """

    pass


def verify_redaction_audit_callback_contract() -> None:
    """Declare the static, unit, and CB-stage proof for the shell boundary.

    The verifier design is ``bash -n acceptance/gcp/scripts/verify-lumen-auth.sh``
    plus ``python3 -m unittest discover -s apps/lumen/external-contracts/tests/unit``;
    fresh GKE evidence is deferred to
    ``aw ec verify --project lumen --stage cb --wi 2879`` after code generation.
    Direct GKE invocation remains valid when both EC variables are absent.
    """

    pass
