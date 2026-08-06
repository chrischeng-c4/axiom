from __future__ import annotations

from dataclasses import dataclass
from typing import Final

from service_k8s.domain.digest import hex_sha256
from service_k8s.domain.purpose import ExtendedUsage, Purpose
from service_k8s.domain.scope import InstanceScope

MIN_LIFETIME_SECS: Final[int] = 300
MAX_LIFETIME_SECS: Final[int] = 604800
MIN_RENEW_BEFORE_SECS: Final[int] = 600
CLUSTER_SUFFIXES: Final[tuple[str, ...]] = (".svc.cluster.local", ".svc")


@dataclass(frozen=True)
class CertificateIdentity:
    dns_names: tuple[str, ...] = ()
    spiffe_uri: str | None = None


class ProfileError(Exception):
    """Base exception for certificate profile validation failures."""


class NoNames(ProfileError):
    def __init__(self) -> None:
        super().__init__("a certificate profile must request at least one DNS name")


class PublicDnsName(ProfileError):
    def __init__(self, name: str) -> None:
        self.name = name
        super().__init__(f"DNS name {name} is not a Kubernetes-internal name")


class ForeignDnsName(ProfileError):
    def __init__(self, name: str, namespace: str) -> None:
        self.name = name
        self.namespace = namespace
        super().__init__(f"DNS name {name} is not inside namespace {namespace}")


class PeerNeedsSpiffeUri(ProfileError):
    def __init__(self) -> None:
        super().__init__("a peer profile must carry a SPIFFE URI")


class ForeignSpiffeUri(ProfileError):
    def __init__(self, uri: str, expected_prefix: str) -> None:
        self.uri = uri
        self.expected_prefix = expected_prefix
        super().__init__(
            f"SPIFFE URI {uri} is outside this instance's scope; it must begin with {expected_prefix}"
        )


class LifetimeOutOfBounds(ProfileError):
    def __init__(self, seconds: int) -> None:
        self.seconds = seconds
        super().__init__(
            f"leaf lifetime {seconds}s is outside {MIN_LIFETIME_SECS}s..{MAX_LIFETIME_SECS}s"
        )


class RenewWindowTooNarrow(ProfileError):
    def __init__(self, renew_before_secs: int) -> None:
        self.renew_before_secs = renew_before_secs
        super().__init__(
            f"renew_before {renew_before_secs}s leaves no room to retry; the floor is {MIN_RENEW_BEFORE_SECS}s"
        )


class RenewWindowTooWide(ProfileError):
    def __init__(self, renew_before_secs: int, lifetime_secs: int) -> None:
        self.renew_before_secs = renew_before_secs
        self.lifetime_secs = lifetime_secs
        super().__init__(
            f"renew_before {renew_before_secs}s is not shorter than the {lifetime_secs}s lifetime"
        )


class JitterExceedsWindow(ProfileError):
    def __init__(self, jitter_secs: int, renew_before_secs: int) -> None:
        self.jitter_secs = jitter_secs
        self.renew_before_secs = renew_before_secs
        super().__init__(
            f"renew jitter {jitter_secs}s exceeds the {renew_before_secs}s renewal window"
        )


@dataclass(frozen=True)
class CertificateProfile:
    scope: InstanceScope
    purpose: Purpose
    common_name: str
    identity: CertificateIdentity
    lifetime_secs: int
    renew_before_secs: int
    renew_jitter_secs: int

    def __post_init__(self) -> None:
        if not self.identity.dns_names:
            raise NoNames()

        for name in self.identity.dns_names:
            if not any(name.endswith(suffix) for suffix in CLUSTER_SUFFIXES):
                raise PublicDnsName(name)
            namespaced = "." + self.scope.namespace + ".svc"
            if namespaced not in name:
                raise ForeignDnsName(name, self.scope.namespace)

        if self.purpose is Purpose.PEER and self.identity.spiffe_uri is None:
            raise PeerNeedsSpiffeUri()
        if self.identity.spiffe_uri is not None:
            prefix = self.scope.spiffe_prefix()
            if not self.identity.spiffe_uri.startswith(prefix):
                raise ForeignSpiffeUri(self.identity.spiffe_uri, prefix)

        if not (MIN_LIFETIME_SECS <= self.lifetime_secs <= MAX_LIFETIME_SECS):
            raise LifetimeOutOfBounds(self.lifetime_secs)

        if self.renew_before_secs < MIN_RENEW_BEFORE_SECS:
            raise RenewWindowTooNarrow(self.renew_before_secs)

        if self.renew_before_secs >= self.lifetime_secs:
            raise RenewWindowTooWide(self.renew_before_secs, self.lifetime_secs)

        if self.renew_jitter_secs > self.renew_before_secs:
            raise JitterExceedsWindow(self.renew_jitter_secs, self.renew_before_secs)

    def extended_key_usages(self) -> tuple[ExtendedUsage, ...]:
        return self.purpose.extended_key_usages()

    def secret_name(self) -> str:
        return self.scope.secret_name(self.purpose)

    def identity_digest(self) -> str:
        parts = [
            "purpose=" + self.purpose.token,
            "cn=" + self.common_name,
            "dns=" + ",".join(sorted(self.identity.dns_names)),
            "uri=" + (self.identity.spiffe_uri or ""),
            "eku=" + ",".join(u.token for u in self.extended_key_usages()),
        ]
        return hex_sha256("|".join(parts).encode("utf-8"))
