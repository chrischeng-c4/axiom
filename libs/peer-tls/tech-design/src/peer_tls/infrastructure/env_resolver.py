"""Environment variable resolver adapter for peer TLS."""

from __future__ import annotations

from dataclasses import dataclass

from peer_tls.infrastructure.ports import EnvironmentSource


@dataclass(frozen=True)
class MaterialLocations:
    leaf: str
    key: str
    trust: str


class EnvPrefixError(Exception):
    pass


def resolve_locations(
    env: EnvironmentSource,
    prefix: str,
) -> MaterialLocations | None:
    cert = env.get(f"{prefix}_CERT")
    key = env.get(f"{prefix}_KEY")
    ca = env.get(f"{prefix}_CA")

    if cert is None and key is None and ca is None:
        return None

    if cert is None or key is None or ca is None:
        missing: list[str] = []
        if cert is None:
            missing.append(f"{prefix}_CERT")
        if key is None:
            missing.append(f"{prefix}_KEY")
        if ca is None:
            missing.append(f"{prefix}_CA")
        raise EnvPrefixError(f"Missing environment variables for prefix '{prefix}': {', '.join(missing)}")

    return MaterialLocations(leaf=cert, key=key, trust=ca)
