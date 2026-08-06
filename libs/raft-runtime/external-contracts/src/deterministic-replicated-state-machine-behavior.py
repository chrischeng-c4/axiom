from __future__ import annotations

from raft_runtime.application.replication import (
    apply_committed,
    replay_plan,
)
from raft_runtime.application.store import (
    INITIAL_HARD_STATE,
    HardState,
    RaftStore,
)
from raft_runtime.infrastructure.applied_index_file import (
    decode_applied_index,
    encode_applied_index,
)

MINIMUM_CHECKS = 12

DETERMINISTIC_REPLICATED_STATE_MACHINE_BEHAVIOR_MATRIX = (
    ("the_replay_plan_drops_every_index_at_or_below_the_applied_floor",
     ((4, 5), (1, 2), ())),
    ("the_replay_plan_deduplicates_and_runs_oldest_first",
     ((1, 2, 3), (2, 4))),
    ("a_fully_applied_or_empty_commit_set_replays_nothing",
     ((), (), ())),
    ("committed_entries_reach_the_machine_in_ascending_index_order",
     ((1, 2, 3), (1, 2, 3), 3)),
    ("the_machines_own_applied_floor_decides_what_is_skipped",
     ((3, 4), (1, 2), (), (3, 4))),
    ("a_repeated_index_inside_one_batch_reaches_the_machine_once",
     ((1, 2), (1,), ((1, 'first'), (2, 'next')))),
    ("an_apply_that_raises_is_still_counted_as_applied_and_named_failed",
     ((1, 2, 3), (2,), (), (1, 2, 3))),
    ("a_restart_replays_the_committed_log_and_reproduces_the_state",
     ('1:a|2:b|3:c', '1:a|2:b|3:c', True, ())),
    ("a_missing_or_empty_applied_index_file_reads_as_a_floor_of_zero",
     (0, 0, 0, 0)),
    ("a_stored_floor_round_trips_and_tolerates_surrounding_whitespace",
     ('0', '7', 12345, 42)),
    ("a_fresh_store_loads_the_initial_hard_state_and_a_saved_one_returns",
     ((None, (0, None)), (0, None), (3, 5), (3, 5))),
    ("only_a_changed_hard_state_is_written_and_every_field_counts",
     (True, False, True, True, (2, 7))),
)


def plain(value: object) -> object:
    """A literal-shaped view: records by their fields, enum members by value.

    An expected value has to be a plain literal, and `repr` of a dataclass or
    an enum member is not one. Reading a record as the tuple of its fields
    keeps every field observable while staying transcribable.
    """
    fields = getattr(type(value), "__dataclass_fields__", None)
    if fields is not None:
        return tuple(plain(getattr(value, n)) for n in fields)
    if getattr(type(value), "__members__", None) is not None:
        return plain(value.value)
    if isinstance(value, tuple):
        return tuple(plain(v) for v in value)
    if isinstance(value, list):
        return [plain(v) for v in value]
    if isinstance(value, dict):
        return {k: plain(v) for k, v in value.items()}
    return value


class Recorder:
    """A state machine that records the order it was asked to apply in.

    It is the only witness of what actually reached the machine: the report
    says what the host decided, and this log says what the machine saw.
    """

    def __init__(self, floor: int = 0, raise_on: tuple[int, ...] = ()) -> None:
        self._floor = floor
        self._raise_on = raise_on
        self.log: list[tuple[int, bytes]] = []
        self.restored: bytes | None = None

    def apply(self, index: int, command: bytes) -> None:
        self.log.append((index, command))
        if index in self._raise_on:
            raise ValueError("the state machine refused this command")
        self._floor = index

    def snapshot(self) -> bytes:
        return b"|".join(b"%d:%s" % (i, c) for i, c in self.log)

    def restore(self, blob: bytes) -> None:
        self.restored = blob

    def applied_index(self) -> int:
        return self._floor

    def indices(self) -> tuple[int, ...]:
        return tuple(i for i, _ in self.log)

    def seen(self) -> tuple[tuple[int, str], ...]:
        """The log as text, because an observation has to survive JSON."""
        return tuple((i, c.decode("ascii")) for i, c in self.log)

    def state(self) -> str:
        return self.snapshot().decode("ascii")


def verify_deterministic_replicated_state_machine_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. the replay plan drops every index at or below the applied floor
    exp1 = DETERMINISTIC_REPLICATED_STATE_MACHINE_BEHAVIOR_MATRIX[0][1]
    obs1 = plain((replay_plan(3, [1, 2, 3, 4, 5]), replay_plan(0, [1, 2]),
        replay_plan(5, [5])))
    checks.append({"name": DETERMINISTIC_REPLICATED_STATE_MACHINE_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. the replay plan deduplicates and runs oldest first
    exp2 = DETERMINISTIC_REPLICATED_STATE_MACHINE_BEHAVIOR_MATRIX[1][1]
    obs2 = plain((replay_plan(0, [3, 1, 2, 1, 3]), replay_plan(1, [4, 2,
        4])))
    checks.append({"name": DETERMINISTIC_REPLICATED_STATE_MACHINE_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. a fully applied or empty commit set replays nothing
    exp3 = DETERMINISTIC_REPLICATED_STATE_MACHINE_BEHAVIOR_MATRIX[2][1]
    obs3 = plain((replay_plan(0, []), replay_plan(9, [1, 2, 3]),
        replay_plan(9, [])))
    checks.append({"name": DETERMINISTIC_REPLICATED_STATE_MACHINE_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. committed entries reach the machine in ascending index order
    exp4 = DETERMINISTIC_REPLICATED_STATE_MACHINE_BEHAVIOR_MATRIX[3][1]
    shuffled = Recorder()
    report = apply_committed(shuffled, [(3, b"c"), (1, b"a"), (2, b"b")])
    obs4 = plain((shuffled.indices(), report.applied,
        shuffled.applied_index()))
    checks.append({"name": DETERMINISTIC_REPLICATED_STATE_MACHINE_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. the machines own applied floor decides what is skipped
    exp5 = DETERMINISTIC_REPLICATED_STATE_MACHINE_BEHAVIOR_MATRIX[4][1]
    resumed = Recorder(floor=2)
    fenced = apply_committed(resumed, [(1, b"a"), (2, b"b"), (3, b"c"), (4, b"d")])
    obs5 = plain((fenced.applied, fenced.skipped, fenced.failed,
        resumed.indices()))
    checks.append({"name": DETERMINISTIC_REPLICATED_STATE_MACHINE_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. a repeated index inside one batch reaches the machine once
    exp6 = DETERMINISTIC_REPLICATED_STATE_MACHINE_BEHAVIOR_MATRIX[5][1]
    deduped = Recorder()
    doubled = apply_committed(deduped, [(1, b"first"), (1, b"second"), (2, b"next")])
    obs6 = plain((doubled.applied, doubled.skipped, deduped.seen()))
    checks.append({"name": DETERMINISTIC_REPLICATED_STATE_MACHINE_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. an apply that raises is still counted as applied and named failed
    exp7 = DETERMINISTIC_REPLICATED_STATE_MACHINE_BEHAVIOR_MATRIX[6][1]
    refusing = Recorder(raise_on=(2,))
    diverged = apply_committed(refusing, [(1, b"a"), (2, b"b"), (3, b"c")])
    obs7 = plain((diverged.applied, diverged.failed, diverged.skipped,
        refusing.indices()))
    checks.append({"name": DETERMINISTIC_REPLICATED_STATE_MACHINE_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. a restart replays the committed log and reproduces the state
    exp8 = DETERMINISTIC_REPLICATED_STATE_MACHINE_BEHAVIOR_MATRIX[7][1]
    live = Recorder()
    apply_committed(live, [(1, b"a"), (2, b"b"), (3, b"c")])
    restarted = Recorder()
    apply_committed(restarted, [(1, b"a"), (2, b"b"), (3, b"c")])
    obs8 = plain((live.state(), restarted.state(), live.snapshot() ==
        restarted.snapshot(), replay_plan(restarted.applied_index(), [1,
        2, 3])))
    checks.append({"name": DETERMINISTIC_REPLICATED_STATE_MACHINE_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. a missing or empty applied index file reads as a floor of zero
    exp9 = DETERMINISTIC_REPLICATED_STATE_MACHINE_BEHAVIOR_MATRIX[8][1]
    obs9 = plain((decode_applied_index(None), decode_applied_index(b""),
        decode_applied_index(b"   "), decode_applied_index(b"\n")))
    checks.append({"name": DETERMINISTIC_REPLICATED_STATE_MACHINE_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. a stored floor round trips and tolerates surrounding whitespace
    exp10 = DETERMINISTIC_REPLICATED_STATE_MACHINE_BEHAVIOR_MATRIX[9][1]
    obs10 = plain((encode_applied_index(0).decode("ascii"),
        encode_applied_index(7).decode("ascii"),
        decode_applied_index(encode_applied_index(12345)),
        decode_applied_index(b" 42\n")))
    checks.append({"name": DETERMINISTIC_REPLICATED_STATE_MACHINE_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. a fresh store loads the initial hard state and a saved one returns
    exp11 = DETERMINISTIC_REPLICATED_STATE_MACHINE_BEHAVIOR_MATRIX[10][1]
    kept = RaftStore()
    before = (kept.last_saved(), kept.load())
    kept.save(HardState(term=3, voted_for=5))
    obs11 = plain((before, INITIAL_HARD_STATE, kept.load(),
        kept.last_saved()))
    checks.append({"name": DETERMINISTIC_REPLICATED_STATE_MACHINE_BEHAVIOR_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. only a changed hard state is written and every field counts
    exp12 = DETERMINISTIC_REPLICATED_STATE_MACHINE_BEHAVIOR_MATRIX[11][1]
    ledger = RaftStore()
    first = ledger.save(HardState(term=1, voted_for=None))
    again = ledger.save(HardState(term=1, voted_for=None))
    vote = ledger.save(HardState(term=1, voted_for=7))
    term = ledger.save(HardState(term=2, voted_for=7))
    obs12 = plain((first, again, vote, term, ledger.last_saved()))
    checks.append({"name": DETERMINISTIC_REPLICATED_STATE_MACHINE_BEHAVIOR_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    return {
        "case_id": "deterministic-replicated-state-machine-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
