"""Volume resize planning and quantity parsing for service-k8s.

StatefulSet volumeClaimTemplates are immutable after creation, so updating
storage declarations in a manifest does not resize existing volumes. This
module detects storage size mismatches, parses Kubernetes storage quantities,
and plans grow-only volume patches, reporting shrink requests as unsupported
because Kubernetes cannot shrink a bound PVC.
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Callable, Final, Mapping


class QuantityError(ValueError):
    """A string that is not a Kubernetes storage quantity."""


SUFFIXES: Final[tuple[tuple[str, int], ...]] = (
    ("Ei", 1152921504606846976),
    ("Pi", 1125899906842624),
    ("Ti", 1099511627776),
    ("Gi", 1073741824),
    ("Mi", 1048576),
    ("Ki", 1024),
    ("E", 1000000000000000000),
    ("P", 1000000000000000),
    ("T", 1000000000000),
    ("G", 1000000000),
    ("M", 1000000),
    ("k", 1000),
)

SHRINK_DETAIL: Final[str] = (
    "desired size is smaller than current; Kubernetes cannot shrink a bound "
    "PVC, recreate it instead"
)


def _int_from_digits(s: str) -> int:
    if not s or not all(c in "0123456789" for c in s):
        raise QuantityError("unrecognized storage quantity " + repr(s))
    return int(s)


def parse_storage_bytes(qty: str) -> int:
    s = qty.strip()
    if s == "":
        raise QuantityError("empty storage quantity")
    for suffix, multiplier in SUFFIXES:
        if s.endswith(suffix):
            head = s[: len(s) - len(suffix)].strip()
            try:
                value = float(head)
            except ValueError:
                raise QuantityError(
                    "invalid numeric part in storage quantity " + repr(qty)
                )
            if value < 0.0:
                raise QuantityError("negative storage quantity " + repr(qty))
            return round(value * multiplier)
    return _int_from_digits(s)


class ResizeKind(Enum):
    GROW = "grow"
    NOOP = "noop"
    SHRINK_UNSUPPORTED = "shrink-unsupported"
    UNPARSEABLE = "unparseable"


@dataclass(frozen=True)
class ResizeAction:
    kind: ResizeKind
    current_bytes: int | None = None
    desired_bytes: int | None = None
    detail: str = ""


def decide(current: str, desired: str) -> ResizeAction:
    try:
        current_bytes = parse_storage_bytes(current)
    except QuantityError as exc:
        return ResizeAction(
            ResizeKind.UNPARSEABLE,
            detail="current quantity " + repr(current) + ": " + str(exc),
        )
    try:
        desired_bytes = parse_storage_bytes(desired)
    except QuantityError as exc:
        return ResizeAction(
            ResizeKind.UNPARSEABLE,
            detail="desired quantity " + repr(desired) + ": " + str(exc),
        )

    if desired_bytes > current_bytes:
        return ResizeAction(
            ResizeKind.GROW,
            current_bytes=current_bytes,
            desired_bytes=desired_bytes,
        )
    if desired_bytes == current_bytes:
        return ResizeAction(
            ResizeKind.NOOP,
            current_bytes=current_bytes,
            desired_bytes=desired_bytes,
            detail="already at desired size",
        )
    return ResizeAction(
        ResizeKind.SHRINK_UNSUPPORTED,
        current_bytes=current_bytes,
        desired_bytes=desired_bytes,
        detail=SHRINK_DETAIL,
    )


@dataclass(frozen=True)
class PvcFacts:
    name: str
    current: str
    storage_class: str | None = None


@dataclass(frozen=True)
class PvcResizeOutcome:
    pvc_name: str
    current: str
    desired: str
    patched: bool
    detail: str


def storage_patch(desired: str) -> dict[str, object]:
    return {"spec": {"resources": {"requests": {"storage": desired}}}}


def plan_resize(
    pvcs: tuple[PvcFacts, ...],
    name_filter: Callable[[str], bool],
    desired_storage: Callable[[str], str],
    allow_expansion: Mapping[str, bool],
    dry_run: bool,
) -> tuple[PvcResizeOutcome, ...]:
    outcomes: list[PvcResizeOutcome] = []
    for pvc in pvcs:
        if not name_filter(pvc.name):
            continue

        desired = desired_storage(pvc.name)
        action = decide(pvc.current, desired)

        if action.kind == ResizeKind.UNPARSEABLE:
            outcomes.append(
                PvcResizeOutcome(
                    pvc_name=pvc.name,
                    current=pvc.current,
                    desired=desired,
                    patched=False,
                    detail=action.detail,
                )
            )
        elif action.kind == ResizeKind.NOOP:
            outcomes.append(
                PvcResizeOutcome(
                    pvc_name=pvc.name,
                    current=pvc.current,
                    desired=desired,
                    patched=False,
                    detail="already at desired size",
                )
            )
        elif action.kind == ResizeKind.SHRINK_UNSUPPORTED:
            outcomes.append(
                PvcResizeOutcome(
                    pvc_name=pvc.name,
                    current=pvc.current,
                    desired=desired,
                    patched=False,
                    detail=SHRINK_DETAIL,
                )
            )
        else:
            if pvc.storage_class is None:
                expandable = False
                class_name = "<none>"
            else:
                class_name = pvc.storage_class
                expandable = allow_expansion.get(class_name, False)

            if not expandable:
                detail = (
                    "StorageClass "
                    + repr(class_name)
                    + " does not allow volume expansion; recreate the PVC/StatefulSet manually"
                )
                outcomes.append(
                    PvcResizeOutcome(
                        pvc_name=pvc.name,
                        current=pvc.current,
                        desired=desired,
                        patched=False,
                        detail=detail,
                    )
                )
            elif dry_run:
                outcomes.append(
                    PvcResizeOutcome(
                        pvc_name=pvc.name,
                        current=pvc.current,
                        desired=desired,
                        patched=False,
                        detail="dry run: would patch spec.resources.requests.storage",
                    )
                )
            else:
                outcomes.append(
                    PvcResizeOutcome(
                        pvc_name=pvc.name,
                        current=pvc.current,
                        desired=desired,
                        patched=True,
                        detail="patched spec.resources.requests.storage",
                    )
                )

    return tuple(outcomes)
