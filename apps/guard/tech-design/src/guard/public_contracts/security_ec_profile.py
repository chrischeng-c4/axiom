"""Public TD boundary for lifecycle-consumable Guard evidence."""

__aw_artifact_id__ = "artifact:guard/security-ec-profile"
__aw_public_contract__ = True


def aw_health_security_metric() -> str:
    return "status, exit code, findings, completion, and prompt change together"


def ec_security_evidence_command() -> str:
    return "adapter exit, report, and findings fold independently and fail closed"


def security_report_consumer_contract() -> str:
    return "public report fields derive one unambiguous lifecycle decision"


def stable_security_metric_projection() -> str:
    return "equivalent scans preserve the path-independent lifecycle metric"
