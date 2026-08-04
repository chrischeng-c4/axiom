"""Tech design for WI #3365: peer-tls: author full-typed Python TD (DDD) and fill the Python EC.

@spec #3365
"""

from __future__ import annotations

__aw_artifact_id__ = "artifact:verifiable-design-lifecycle-for-peer-tls/author-full-typed-python-td-ddd-and-fill-the-python-ec-wi-3365"
__aw_work_item__ = "3365"


def design_contract() -> str:
    """Express the executable design contract for this bounded change."""
    return "material-validation, mtls-config-construction, rotation-and-reload"


def domain_modules() -> tuple[str, ...]:
    return (
        "peer_tls.domain.identity",
        "peer_tls.domain.material",
        "peer_tls.domain.verdict",
        "peer_tls.domain.validation",
        "peer_tls.domain.rotation",
    )


def application_modules() -> tuple[str, ...]:
    return (
        "peer_tls.application.validate_material",
        "peer_tls.application.build_mtls_config",
        "peer_tls.application.rotate_material",
    )


def infrastructure_modules() -> tuple[str, ...]:
    return (
        "peer_tls.infrastructure.ports",
        "peer_tls.infrastructure.env_resolver",
        "peer_tls.infrastructure.config_plan",
    )
