"""Public TD boundary for Guard's static security scan."""

__aw_artifact_id__ = "artifact:guard/static-security-scan"
__aw_public_contract__ = True


def compass_backed_diagnostic_scan() -> str:
    return "Compass diagnostics retain rule, source, location, and count"


def json_report_envelope() -> str:
    return "clean scans emit a self-consistent guard.report/1 envelope"


def scan_command_report_projection() -> str:
    return "scan projects its target, policy, engine, and completion publicly"


def stable_static_finding_normalization() -> str:
    return "equivalent diagnostics preserve path-independent normalized fields"
