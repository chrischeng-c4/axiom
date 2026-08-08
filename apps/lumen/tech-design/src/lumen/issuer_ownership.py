"""Closed typed ownership boundary for externally provisioned Lumen TLS."""
from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Final

__aw_artifact_id__: Final[str] = "artifact:lumen/issuer-ownership"


class Classification(Enum):
    OPERATOR_INPUT = "operator-input"
    EXTERNAL_SECRET = "external-secret"
    RETIRED_FORBIDDEN = "retired-forbidden"


@dataclass(frozen=True)
class Check:
    surface: str
    expected: Classification
    observed: Classification


@dataclass(frozen=True)
class Report:
    checks: tuple[Check, ...]

    @property
    def passed(self) -> bool:
        return bool(self.checks) and all(
            check.expected is check.observed for check in self.checks
        )


EXPECTED_CLASSIFICATIONS: Final[dict[str, Classification]] = {
    "namespace": Classification.OPERATOR_INPUT,
    "image": Classification.OPERATOR_INPUT,
    "monitoring": Classification.OPERATOR_INPUT,
    "servingTlsSecret": Classification.EXTERNAL_SECRET,
    "peerTlsSecret": Classification.EXTERNAL_SECRET,
    "--issuer": Classification.RETIRED_FORBIDDEN,
    "--trust-domain": Classification.RETIRED_FORBIDDEN,
    "--ca-pool": Classification.RETIRED_FORBIDDEN,
    "LUMEN_ISSUER": Classification.RETIRED_FORBIDDEN,
    "LUMEN_TRUST_DOMAIN": Classification.RETIRED_FORBIDDEN,
    "LUMEN_CA_POOL": Classification.RETIRED_FORBIDDEN,
    "cas-resolver": Classification.RETIRED_FORBIDDEN,
    "metadata-token-source": Classification.RETIRED_FORBIDDEN,
    "cas": Classification.RETIRED_FORBIDDEN,
    "ephemeral": Classification.RETIRED_FORBIDDEN,
}


def classify(surface: str) -> Classification:
    """Classify one known surface; unknown surfaces fail closed."""
    try:
        return EXPECTED_CLASSIFICATIONS[surface]
    except KeyError as exc:
        raise ValueError(f"unknown issuer ownership surface: {surface}") from exc


def evaluate() -> Report:
    """Evaluate the complete frozen matrix against the exact classifier."""
    return Report(tuple(
        Check(surface, expected, classify(surface))
        for surface, expected in EXPECTED_CLASSIFICATIONS.items()
    ))
