from __future__ import annotations

from dataclasses import dataclass

from service_k8s.domain.purpose import Purpose


@dataclass(frozen=True)
class InstanceScope:
    namespace: str
    instance: str
    trust_domain: str

    def secret_name(self, purpose: Purpose) -> str:
        return f"{self.instance}-{purpose.token}-tls"

    def spiffe_prefix(self) -> str:
        return f"spiffe://{self.trust_domain}/ns/{self.namespace}/"

    def covers(self, other: InstanceScope) -> bool:
        return self == other
