from __future__ import annotations

from raft_runtime.domain.errors import (
    AlreadyAssigned,
    AssignmentError,
    Expired,
    ExpiryNotLater,
    ExpiryNotInFuture,
    OwnerMismatch,
    StaleEpoch,
    Unassigned,
)
from raft_runtime.domain.fencing import (
    ActiveAssignment,
    AssignmentEpoch,
    FenceToken,
    fence_problem,
    is_expired,
    next_epoch,
)


class FencedAssignment:
    def __init__(self) -> None:
        self._epoch: AssignmentEpoch = 0
        self._active: ActiveAssignment | None = None

    def epoch(self) -> AssignmentEpoch:
        return self._epoch

    def idle(self) -> bool:
        return self._active is None

    def active(self) -> ActiveAssignment | None:
        return self._active

    def token(self) -> FenceToken | None:
        return self._active.token if self._active is not None else None

    def assign(
        self, owner: str, expires_at_ms: int, now_ms: int
    ) -> FenceToken | AssignmentError:
        if expires_at_ms <= now_ms:
            return ExpiryNotInFuture(
                expires_at_ms=expires_at_ms, now_ms=now_ms
            )

        if self._active is not None and not is_expired(
            self._active.expires_at_ms, now_ms
        ):
            return AlreadyAssigned(
                owner=self._active.token.owner, epoch=self._active.token.epoch
            )

        self._epoch = next_epoch(self._epoch)
        self._active = ActiveAssignment(
            token=FenceToken(owner=owner, epoch=self._epoch),
            expires_at_ms=expires_at_ms,
        )
        return self._active.token

    def validate(
        self, owner: str, epoch: AssignmentEpoch, now_ms: int
    ) -> AssignmentError | None:
        return fence_problem(
            active=self._active, owner=owner, epoch=epoch, now_ms=now_ms
        )

    def renew(
        self, owner: str, epoch: AssignmentEpoch, expires_at_ms: int, now_ms: int
    ) -> ActiveAssignment | AssignmentError:
        problem = self.validate(owner=owner, epoch=epoch, now_ms=now_ms)
        if problem is not None:
            return problem

        assert self._active is not None
        if expires_at_ms <= self._active.expires_at_ms:
            return ExpiryNotLater(
                current_ms=self._active.expires_at_ms, supplied=expires_at_ms
            )

        self._active = ActiveAssignment(
            token=self._active.token, expires_at_ms=expires_at_ms
        )
        return self._active

    def release(
        self, owner: str, epoch: AssignmentEpoch
    ) -> AssignmentError | None:
        if self._active is None:
            return Unassigned()
        if epoch != self._active.token.epoch:
            return StaleEpoch(expected=self._active.token.epoch, supplied=epoch)
        if owner != self._active.token.owner:
            return OwnerMismatch(
                expected=self._active.token.owner, supplied=owner
            )

        self._active = None
        return None

    def expire(self, now_ms: int) -> bool:
        if self._active is not None and is_expired(
            self._active.expires_at_ms, now_ms
        ):
            self._active = None
            return True
        return False
