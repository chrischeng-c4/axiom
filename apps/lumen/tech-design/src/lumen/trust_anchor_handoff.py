"""Closed typed contract for the externally distributed serving trust anchor."""
from __future__ import annotations

from enum import Enum
from typing import Final

__aw_artifact_id__: Final[str] = "artifact:lumen/trust-anchor-handoff"


class Classification(Enum):
    CLIENT_INPUT = "client-input"
    EXTERNAL_HANDOFF = "external-handoff"
    FORBIDDEN_PUBLISHER = "forbidden-publisher"


EXPECTED_CLASSIFICATIONS: Final[dict[str, Classification]] = {
    "--ca-file": Classification.CLIENT_INPUT,
    "PrivateTrust": Classification.CLIENT_INPUT,
    "servingTlsSecret": Classification.EXTERNAL_HANDOFF,
    "public-ca": Classification.EXTERNAL_HANDOFF,
    "ConfigMap-publisher": Classification.FORBIDDEN_PUBLISHER,
    "status-discovery": Classification.FORBIDDEN_PUBLISHER,
    "pod-kubernetes-writer": Classification.FORBIDDEN_PUBLISHER,
    "trust-bundle-Role": Classification.FORBIDDEN_PUBLISHER,
    "trust-bundle-RoleBinding": Classification.FORBIDDEN_PUBLISHER,
    "automatic-ca-publication": Classification.FORBIDDEN_PUBLISHER,
}


def classify(surface: str) -> Classification:
    """Classify a known handoff surface and reject unknown additions."""
    try:
        return EXPECTED_CLASSIFICATIONS[surface]
    except KeyError as exc:
        raise ValueError(f"unknown trust-anchor handoff surface: {surface}") from exc
