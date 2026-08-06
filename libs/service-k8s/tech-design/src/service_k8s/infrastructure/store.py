"""Store edge logic and Server-Side Apply patch preparation for service-k8s.

The store edge is restricted to read and apply operations (no delete) so that
an error path cannot accidentally purge secrets containing certificate material.
This module classifies API status and transport errors (redacting error messages)
and prepares Server-Side Apply patches with back-filled certificate data to
prevent field-manager field pruning during trust bundle updates.
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Final

from service_k8s.application.status import redact
from service_k8s.infrastructure.projection import (
    CERT_KEY,
    IDENTITY_DIGEST_ANNOTATION,
    LEAF_ISSUER_ANNOTATION,
    PRIVATE_KEY_KEY,
    TRUST_BUNDLE_ANNOTATION,
    TRUST_BUNDLE_KEY,
)

FIELD_MANAGER: Final[str] = "service-k8s-certificate"

# The lifecycle requires only get and patch operations. No delete, create,
# update, list, or watch verbs are used.
REQUIRED_RBAC_VERBS: Final[tuple[str, ...]] = ("get", "patch")

LIFECYCLE_DATA_KEYS: Final[tuple[str, ...]] = (
    CERT_KEY,
    PRIVATE_KEY_KEY,
    TRUST_BUNDLE_KEY,
)

LIFECYCLE_ANNOTATION_KEYS: Final[tuple[str, ...]] = (
    TRUST_BUNDLE_ANNOTATION,
    LEAF_ISSUER_ANNOTATION,
    IDENTITY_DIGEST_ANNOTATION,
)

LIFECYCLE_LABEL_KEYS: Final[tuple[str, ...]] = (
    "app.kubernetes.io/name",
    "app.kubernetes.io/managed-by",
    "app.kubernetes.io/component",
)


class StoreErrorKind(Enum):
    FORBIDDEN = "forbidden"
    CONFLICT = "conflict"
    UNAVAILABLE = "unavailable"
    MALFORMED = "malformed"
    OTHER = "other"


@dataclass(frozen=True)
class StoreError(Exception):
    kind: StoreErrorKind
    message: str
    code: int | None = None

    @staticmethod
    def new(
        kind: StoreErrorKind, message: str, code: int | None = None
    ) -> StoreError:
        return StoreError(kind=kind, message=redact(message), code=code)

    @staticmethod
    def forbidden(message: str) -> StoreError:
        return StoreError.new(StoreErrorKind.FORBIDDEN, message)

    @staticmethod
    def conflict(message: str) -> StoreError:
        return StoreError.new(StoreErrorKind.CONFLICT, message)

    @staticmethod
    def unavailable(message: str) -> StoreError:
        return StoreError.new(StoreErrorKind.UNAVAILABLE, message)

    @staticmethod
    def malformed(message: str) -> StoreError:
        return StoreError.new(StoreErrorKind.MALFORMED, message)

    def retryable(self) -> bool:
        if self.kind == StoreErrorKind.CONFLICT:
            return True
        if self.kind == StoreErrorKind.UNAVAILABLE:
            return True
        if self.kind == StoreErrorKind.FORBIDDEN:
            return False
        if self.kind == StoreErrorKind.MALFORMED:
            return False
        if self.kind == StoreErrorKind.OTHER:
            if self.code is not None and (self.code >= 500 or self.code == 429):
                return True
            return False
        return False


def classify_status(code: int, message: str) -> StoreError:
    if code == 403:
        return StoreError.forbidden(message)
    if code == 409:
        return StoreError.conflict(message)
    if code >= 500:
        return StoreError.unavailable(message)
    return StoreError.new(StoreErrorKind.OTHER, message, code)


def classify_transport(message: str) -> StoreError:
    return StoreError.unavailable(message)


@dataclass(frozen=True)
class PatchDecision:
    patch: dict[str, object]
    unchanged: bool


def prepare_patch(
    desired: dict[str, object], live: dict[str, object] | None
) -> PatchDecision:
    meta = desired.get("metadata")
    if not isinstance(meta, dict):
        raise StoreError.malformed("missing metadata")

    name = meta.get("name")
    if not isinstance(name, str):
        raise StoreError.malformed("missing metadata.name")

    namespace = meta.get("namespace")
    if not isinstance(namespace, str):
        raise StoreError.malformed("missing metadata.namespace")

    raw_data = desired.get("stringData")
    data: dict[str, str] = {}
    if isinstance(raw_data, dict):
        for k, v in raw_data.items():
            if isinstance(v, str):
                data[k] = v

    raw_ann = meta.get("annotations")
    annotations: dict[str, str] = {}
    if isinstance(raw_ann, dict):
        for k, v in raw_ann.items():
            if isinstance(v, str):
                annotations[k] = v

    raw_labels = meta.get("labels")
    labels: dict[str, str] = {}
    if isinstance(raw_labels, dict):
        for k, v in raw_labels.items():
            if isinstance(v, str):
                labels[k] = v

    owners = meta.get("ownerReferences")

    live_data: dict[str, str] = {}
    live_ann: dict[str, str] = {}
    live_labels: dict[str, str] = {}
    live_meta: dict[str, object] | None = None
    live_owners: list[object] | None = None
    if isinstance(live, dict):
        raw_live_meta = live.get("metadata")
        if isinstance(raw_live_meta, dict):
            live_meta = raw_live_meta
            raw_lo = raw_live_meta.get("ownerReferences")
            if isinstance(raw_lo, list):
                live_owners = raw_lo
            raw_la = raw_live_meta.get("annotations")
            if isinstance(raw_la, dict):
                for k, v in raw_la.items():
                    if isinstance(v, str):
                        live_ann[k] = v
            raw_lbl = raw_live_meta.get("labels")
            if isinstance(raw_lbl, dict):
                for k, v in raw_lbl.items():
                    if isinstance(v, str):
                        live_labels[k] = v

        raw_ld = live.get("data")
        if isinstance(raw_ld, dict):
            for k, v in raw_ld.items():
                if isinstance(v, str):
                    live_data[k] = v

    if live is not None:
        for key in LIFECYCLE_DATA_KEYS:
            if key not in data and key in live_data:
                data[key] = live_data[key]
        for key in LIFECYCLE_ANNOTATION_KEYS:
            if key not in annotations and key in live_ann:
                annotations[key] = live_ann[key]

    patch_meta: dict[str, object] = {
        "name": name,
        "namespace": namespace,
        "labels": labels,
        "annotations": annotations,
    }
    if owners is not None:
        patch_meta["ownerReferences"] = owners

    patch: dict[str, object] = {
        "apiVersion": "v1",
        "kind": "Secret",
        "type": "Opaque",
        "metadata": patch_meta,
        "stringData": data,
    }

    if live is None:
        return PatchDecision(patch, unchanged=False)

    unchanged = True

    if live_meta is None:
        unchanged = False
    else:
        if live_meta.get("name") != name:
            unchanged = False
        if live_meta.get("namespace") != namespace:
            unchanged = False
    if live.get("type") != "Opaque":
        unchanged = False

    if owners is not None and isinstance(owners, list):
        if not isinstance(live_owners, list):
            unchanged = False
        else:
            for des_owner in owners:
                if not isinstance(des_owner, dict):
                    unchanged = False
                    break
                match_found = False
                for lo in live_owners:
                    if isinstance(lo, dict):
                        keys = (
                            "apiVersion",
                            "kind",
                            "name",
                            "uid",
                            "controller",
                            "blockOwnerDeletion",
                        )
                        if all(lo.get(k) == des_owner.get(k) for k in keys):
                            match_found = True
                            break
                if not match_found:
                    unchanged = False
                    break
    elif owners is None:
        if live_owners is not None and len(live_owners) > 0:
            unchanged = False

    for key in LIFECYCLE_DATA_KEYS:
        if data.get(key) != live_data.get(key):
            unchanged = False

    for key in LIFECYCLE_ANNOTATION_KEYS:
        if annotations.get(key) != live_ann.get(key):
            unchanged = False

    for key in LIFECYCLE_LABEL_KEYS:
        if labels.get(key) != live_labels.get(key):
            unchanged = False

    return PatchDecision(patch=patch, unchanged=unchanged)
