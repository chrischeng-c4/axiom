"""Application service for building mTLS configuration plans."""

from __future__ import annotations

from dataclasses import dataclass

from peer_tls.domain.identity import IdentityExpectation
from peer_tls.domain.material import MaterialTriple
from peer_tls.domain.validation import decide_material
from peer_tls.domain.verdict import ValidatedMaterial
from peer_tls.infrastructure.config_plan import ClientConfigPlan, ServerConfigPlan
from peer_tls.infrastructure.env_resolver import EnvPrefixError
from peer_tls.infrastructure.ports import Clock, CryptoProviderInstaller, EnvironmentSource


@dataclass(frozen=True)
class BuildMtlsConfigService:
    env: EnvironmentSource
    installer: CryptoProviderInstaller
    clock: Clock

    def execute(
        self,
        prefix: str,
        triple: MaterialTriple,
        expectation: IdentityExpectation,
        leaf_label: str,
    ) -> tuple[ServerConfigPlan, ClientConfigPlan] | None:
        cert = self.env.get(f"{prefix}_CERT")
        key = self.env.get(f"{prefix}_KEY")
        ca = self.env.get(f"{prefix}_CA")

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

        self.installer.install_default()

        verdict = decide_material(triple, expectation, self.clock.now())
        if not isinstance(verdict, ValidatedMaterial):
            return None

        server_plan = ServerConfigPlan(
            trust=triple.trust,
            leaf_label=leaf_label,
        )
        client_plan = ClientConfigPlan(
            trust=triple.trust,
            leaf_label=leaf_label,
        )
        return (server_plan, client_plan)
