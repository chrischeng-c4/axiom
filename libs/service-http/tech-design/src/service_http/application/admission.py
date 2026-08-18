from __future__ import annotations

from dataclasses import dataclass

from service_http.domain.admission import (
    AdmissionPolicy,
    Decision,
    Event,
    Outcome,
    max_credits,
    request_cost,
)


@dataclass(frozen=True)
class BucketKey:
    route_class: str
    fingerprint: str


@dataclass
class Bucket:
    credits: int
    last_ns: int
    last_seen: int


class AdmissionLedger:
    def __init__(self, policies: dict[str, AdmissionPolicy]) -> None:
        self._policies: dict[str, AdmissionPolicy] = dict(policies)
        self._buckets: dict[BucketKey, Bucket] = {}
        self._sequence: int = 0

    def sequence(self) -> int:
        return self._sequence

    def tracked_keys(self, route_class: str) -> int:
        return sum(
            1 for key in self._buckets if key.route_class == route_class
        )

    def total_keys(self) -> int:
        return len(self._buckets)

    def admit_at(
        self, route_class: str, fingerprint: str, now_ns: int
    ) -> Decision:
        policy = self._policies.get(route_class)
        if policy is None:
            return Decision(Outcome.BYPASS, None)

        self._sequence += 1
        key = BucketKey(route_class, fingerprint)
        cap = max_credits(policy)
        cost = request_cost(policy)

        bucket = self._buckets.get(key)
        if bucket is None:
            if self.tracked_keys(route_class) >= policy.max_keys:
                same_class_buckets = [
                    (b.last_seen, k)
                    for k, b in self._buckets.items()
                    if k.route_class == route_class
                ]
                if same_class_buckets:
                    smallest_key = min(same_class_buckets, key=lambda x: x[0])[1]
                    del self._buckets[smallest_key]
            bucket = Bucket(credits=cap, last_ns=now_ns, last_seen=self._sequence)
            self._buckets[key] = bucket
        else:
            elapsed = now_ns - bucket.last_ns
            if elapsed < 0:
                elapsed = 0
            bucket.credits = min(bucket.credits + elapsed * policy.capacity, cap)
            bucket.last_ns = now_ns
            bucket.last_seen = self._sequence

        if bucket.credits >= cost:
            bucket.credits -= cost
            return Decision(Outcome.ALLOW, None)

        missing = cost - bucket.credits
        wait_ns = (missing + policy.capacity - 1) // policy.capacity
        if wait_ns < 1:
            wait_ns = 1
        return Decision(Outcome.DENY, wait_ns)


def decision_event(route_class: str, decision: Decision) -> Event:
    retry_ms = (
        None
        if decision.retry_after_ns is None
        else decision.retry_after_ns // 1_000_000
    )
    return Event(route_class, decision.outcome, retry_ms)
