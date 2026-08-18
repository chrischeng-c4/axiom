from __future__ import annotations

import base64
from dataclasses import dataclass
from datetime import datetime
from typing import Callable, Final, Protocol

from service_k8s.application.rotation import IssuerId, ObservedLeaf
from service_k8s.application.trust_bundle import TrustBundle, split_pem_blocks
from service_k8s.domain.digest import hex_sha256
from service_k8s.domain.purpose import Purpose
from service_k8s.domain.scope import InstanceScope

CERT_KEY: Final[str] = "tls.crt"
PRIVATE_KEY_KEY: Final[str] = "tls.key"
TRUST_BUNDLE_KEY: Final[str] = "ca.crt"

TRUST_BUNDLE_ANNOTATION: Final[str] = "service-k8s.axiom.dev/trust-bundle"
LEAF_ISSUER_ANNOTATION: Final[str] = "service-k8s.axiom.dev/leaf-issuer"
IDENTITY_DIGEST_ANNOTATION: Final[str] = "service-k8s.axiom.dev/identity-digest"

MANAGED_BY: Final[str] = "service-k8s"


@dataclass(frozen=True)
class Owner:
    api_version: str
    kind: str
    name: str
    uid: str

    def reference(self) -> dict[str, object]:
        return {
            "apiVersion": self.api_version,
            "kind": self.kind,
            "name": self.name,
            "uid": self.uid,
            "controller": True,
            "blockOwnerDeletion": True,
        }


def labels(scope: InstanceScope, purpose: Purpose) -> dict[str, str]:
    return {
        "app.kubernetes.io/name": scope.instance,
        "app.kubernetes.io/managed-by": MANAGED_BY,
        "app.kubernetes.io/component": f"{purpose.token}-tls",
    }


# The Secret type is Opaque rather than kubernetes.io/tls because Opaque allows
# publishing trust_bundle_secret (ca.crt alone) during bootstrap before any
# leaf certificate or private key exists.
def base_secret(
    scope: InstanceScope, purpose: Purpose, owner: Owner
) -> dict[str, object]:
    return {
        "apiVersion": "v1",
        "kind": "Secret",
        "type": "Opaque",
        "metadata": {
            "name": scope.secret_name(purpose),
            "namespace": scope.namespace,
            "labels": labels(scope, purpose),
            "ownerReferences": [owner.reference()],
        },
    }


@dataclass(frozen=True)
class IssuedMaterial:
    issuer: IssuerId
    certificate_pem: str
    chain_pem: str
    not_before: datetime
    not_after: datetime
    fingerprint: str


def material_secret(
    scope: InstanceScope,
    purpose: Purpose,
    owner: Owner,
    material: IssuedMaterial,
    private_key_pem: str,
    bundle: TrustBundle,
    identity_digest: str,
) -> dict[str, object]:
    secret = base_secret(scope, purpose, owner)
    secret["metadata"]["annotations"] = {
        TRUST_BUNDLE_ANNOTATION: bundle.annotation(),
        LEAF_ISSUER_ANNOTATION: material.issuer.value,
        IDENTITY_DIGEST_ANNOTATION: identity_digest,
    }
    secret["stringData"] = {
        CERT_KEY: material.certificate_pem,
        PRIVATE_KEY_KEY: private_key_pem,
        TRUST_BUNDLE_KEY: bundle.to_pem(),
    }
    return secret


def trust_bundle_secret(
    scope: InstanceScope,
    purpose: Purpose,
    owner: Owner,
    bundle: TrustBundle,
) -> dict[str, object]:
    secret = base_secret(scope, purpose, owner)
    secret["metadata"]["annotations"] = {
        TRUST_BUNDLE_ANNOTATION: bundle.annotation()
    }
    secret["stringData"] = {TRUST_BUNDLE_KEY: bundle.to_pem()}
    return secret


@dataclass(frozen=True)
class LeafFacts:
    not_before: datetime
    not_after: datetime
    fingerprint: str


class LeafParseError(ValueError):
    """The bytes in tls.crt are not a certificate this module can read."""


class ValidityReader(Protocol):
    def __call__(self, der: bytes) -> tuple[datetime, datetime]:
        ...


def pem_body_to_der(block: str) -> bytes:
    body = "".join(
        line for line in block.splitlines() if not line.startswith("-----")
    )
    try:
        return base64.b64decode(body.strip(), validate=True)
    except Exception:
        raise LeafParseError("decode PEM body")


def parse_leaf(pem: str, read_validity: ValidityReader) -> LeafFacts:
    blocks = split_pem_blocks(pem)
    if not blocks:
        raise LeafParseError("no PEM block")
    der = pem_body_to_der(blocks[0])
    not_before, not_after = read_validity(der)
    return LeafFacts(not_before, not_after, hex_sha256(der))


@dataclass(frozen=True)
class ProjectedState:
    leaf: ObservedLeaf | None = None
    bundle: TrustBundle = TrustBundle()


def read_state(
    data: dict[str, bytes],
    annotations: dict[str, str],
    read_validity: ValidityReader,
) -> ProjectedState:
    bundle = TrustBundle()
    raw = data.get(TRUST_BUNDLE_KEY)
    if raw is not None:
        try:
            text = raw.decode("utf-8")
        except UnicodeDecodeError:
            text = None
        if text is not None:
            bundle = TrustBundle.parse(
                text, annotations.get(TRUST_BUNDLE_ANNOTATION)
            )

    leaf = None
    raw_cert = data.get(CERT_KEY)
    issuer = annotations.get(LEAF_ISSUER_ANNOTATION)
    identity_digest = annotations.get(IDENTITY_DIGEST_ANNOTATION)
    if (
        raw_cert is not None
        and issuer is not None
        and identity_digest is not None
    ):
        try:
            pem = raw_cert.decode("utf-8")
            facts = parse_leaf(pem, read_validity)
        except (UnicodeDecodeError, LeafParseError):
            facts = None
        if facts is not None:
            leaf = ObservedLeaf(
                issuer=IssuerId(issuer),
                not_before=facts.not_before,
                not_after=facts.not_after,
                fingerprint=facts.fingerprint,
                identity_digest=identity_digest,
            )

    return ProjectedState(leaf=leaf, bundle=bundle)
