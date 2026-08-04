from __future__ import annotations

from dataclasses import dataclass

from service_k8s.application.rotation import IssuerId


def split_pem_blocks(pem: str) -> tuple[str, ...]:
    blocks: list[str] = []
    current: list[str] | None = None
    for line in pem.splitlines():
        if line.startswith("-----BEGIN"):
            current = [line]
        elif line.startswith("-----END"):
            if current is not None:
                current.append(line)
                blocks.append("\n".join(current))
                current = None
        elif current is not None:
            current.append(line)
    return tuple(blocks)


@dataclass(frozen=True)
class TrustBundle:
    entries: tuple[tuple[IssuerId, str], ...] = ()

    def with_anchor(self, issuer: IssuerId, anchor_pem: str) -> TrustBundle:
        pairs = [(i, p) for (i, p) in self.entries if i != issuer]
        pairs.append((issuer, anchor_pem))
        pairs.sort(key=lambda pair: pair[0])
        return TrustBundle(tuple(pairs))

    def retaining(self, issuers: tuple[IssuerId, ...]) -> TrustBundle:
        issuers_set = set(issuers)
        pairs = [(i, p) for (i, p) in self.entries if i in issuers_set]
        return TrustBundle(tuple(pairs))

    def issuers(self) -> tuple[IssuerId, ...]:
        return tuple(i for (i, _) in self.entries)

    def contains(self, issuer: IssuerId) -> bool:
        return any(i == issuer for (i, _) in self.entries)

    def is_empty(self) -> bool:
        return len(self.entries) == 0

    def to_pem(self) -> str:
        out = ""
        for _, pem in self.entries:
            out += pem.rstrip()
            out += "\n"
        return out

    def annotation(self) -> str:
        return ",".join(issuer.value for issuer in self.issuers())

    @staticmethod
    def parse(pem: str, annotation: str | None) -> TrustBundle:
        blocks = split_pem_blocks(pem)
        ids = [
            IssuerId(s.strip())
            for s in (annotation or "").split(",")
            if s.strip()
        ]
        if len(blocks) != len(ids):
            return TrustBundle()
        bundle = TrustBundle()
        for id_, block in zip(ids, blocks):
            bundle = bundle.with_anchor(id_, block)
        return bundle
