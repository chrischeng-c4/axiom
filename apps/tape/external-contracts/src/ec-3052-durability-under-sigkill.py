"""EC ec-3052-durability: every acknowledged append survives SIGKILL.

The oracle is the *client's own record* of what each append returned. It is
built before the kill, held in the verifier's memory, and never derived from
anything the server said about its own state. After the kill the server is
restarted from the same `--data-dir` and asked to replay; the recovered set is
compared against that record.

Why this contract exists, and why it must pass both before and after #3052:
#3052 replaces a whole-file `atomic_write` per mutation with an append-only WAL
plus group commit. That is an optimisation of the durability path, and the
failure mode of durability optimisations is that they are fast because they are
no longer durable. Group commit in particular is only correct if the response
is withheld until the barrier covering *that* record returns; get the batching
wrong and every number improves while acknowledged writes start disappearing.
This contract is the thing that would notice. It should be green today, on the
whole-file path, and still green afterwards. A run that only ever happened
after the change would not tell you the oracle works.

### The client records three disjoint outcomes, not two

Every submitted key lands in exactly one of:

- **acked** -- a 2xx came back. The server promised durability. Recovery is
  mandatory.
- **refused** -- a non-2xx came back. The server explicitly declined the write
  (a 507 from `enforce_storage_writable`, a 500 from a failed persist). It
  promised the opposite. Recovery is *forbidden*.
- **unknown** -- no response at all, because the kill landed mid-flight. The
  outcome is genuinely unknown to the client, and recovery may legally go
  either way.

Collapsing `refused` into `unknown` would silently discard the one half of
AC4's second clause that is actually enforceable -- see below. That collapse is
not hypothetical: `urllib.request.urlopen` raises `HTTPError` on every non-2xx,
so until `harness.append` was taught to catch it and return the code, the
`refused` branch below was unreachable, the set was permanently empty, and both
assertions that read it were vacuous while reporting green.

### What SIGKILL establishes, and what it does not

SIGKILL destroys the process. It does not flush or discard the page cache, so
bytes written with `write(2)` and never fsynced are still readable by the next
process -- measured, not assumed: 3200 unsynced bytes survived a SIGKILLed
child intact on this host. Every assertion below therefore verifies crash
consistency across process death: acknowledged records come back, refused ones
do not, nothing is invented, nothing is applied twice. None of them verifies
that a barrier was ever issued; a build that acked before fsync would pass all
of them.

That gap is closed on the efficiency side rather than here, by the barrier
ceiling in `ec-3052-durable-append-scaling.py`: a build that does not wait for
a barrier runs faster than one barrier per in-flight writer allows, which is
arithmetic and needs no fault injection. Splitting it that way is deliberate --
the alternative is a host-specific power-loss rig, and a contract that skips on
the wrong host is a false green wearing a green badge.

### On AC4's exact wording

#3052's AC4 asks for a test that "recovers every acknowledged append and no
unacknowledged one". The first half is a real invariant and is asserted here.
The second half, read literally, is not achievable by any correct
implementation: if the durability barrier succeeds and the process is killed
before the response is written to the socket, the record is legitimately
durable and legitimately unacknowledged. Demanding its absence would demand
that the server un-write data it has already committed. This is not a
theoretical edge -- two runs of this contract against the same unchanged binary
produced `recovered_beyond_acknowledged` of 1 and then 0, so literal AC4 is a
coin flip on code that is behaving correctly.

What *is* enforceable is asserted, in two parts:

- `recovered ∩ refused = ∅` -- a write the server explicitly declined must not
  come back. This is the achievable remainder of AC4's second clause, and it is
  the part a naive "no unacknowledged one" reading and a naive
  "recovered ⊆ sent" reading both miss.

  This one needs `refused` to be non-empty or it asserts nothing, and a healthy
  tape refuses nothing on its own -- so for two revisions the clause was
  satisfied by every green run with `"refused": 0`, vacuously. Worse, the
  precondition below used to abort on *any* refusal, which made the set empty by
  construction: unreachable code guarded by a check that guaranteed its
  unreachability. It is now driven deliberately. `REFUSAL_PROBES` appends are
  submitted with a truncated JSON body before the writers start; the handler
  rejects them at `serde_json::from_slice` with 400, before it touches the
  journal (`apps/tape/src/server.rs:639`). Each probe body still *contains* its
  key as literal bytes, so a server that recovered one would have had to parse
  and store a request it told the client it had refused -- which is exactly the
  lie the assertion is looking for.
- `recovered ⊆ sent` -- no phantom: recovery may not invent a record the client
  never submitted.

The set between `acked` and `sent ∖ refused` -- submitted, killed in flight,
outcome unknown -- is exactly the set that may legally go either way, and this
contract reports its size rather than pretending it is empty.

### Coverage boundary: AC7 is deliberately not here

#3052's AC7 (a fault-injected ENOSPC yields 507 plus sticky degraded read-only)
cannot be driven from outside the process portably. Degraded mode is entered
from exactly one place -- `AppState::persist` discriminating on
`io::ErrorKind::StorageFull` (`apps/tape/src/server.rs:206-214`) -- and there is
no external trigger: the re-probe task only *clears* the flag, and no env var or
admin route sets it. Producing a real ENOSPC needs a size-capped filesystem
(`hdiutil` on macOS, a root-mounted tmpfs on Linux), which would make this
contract host-specific, and a contract that skips on the wrong host is a false
green wearing a green badge. `chmod` does not work either: POSIX permission
changes do not affect an already-open fd, which is what the WAL writer holds.

The refusal probes above are the weaker half of the same idea and should not be
read as covering the stronger one. A malformed body is refused *before* the
handler touches the journal, so recovering one would take an outright parser
bug. The sharp case is a refusal issued *after* the record was applied in
memory -- `journal.append` succeeds, `persist` then fails and the handler
answers 500/507 -- where the write really is sitting in the server's state when
the client is told it was declined. That is the same fail-closed path AC7 names,
it needs the same ENOSPC injection, and under group commit it becomes the
central correctness question rather than an edge: a batch whose barrier fails
must apply none of its members, and nothing outside the process can currently
make that barrier fail.

So AC7 stays a Rust-level test, and the TD owes it a new home: today's coverage
(`server.rs:1671-1714`) is injected *inside* the synchronous `persist` that
#3052 deletes, and group commit moves the error off the request thread, which is
precisely where fail-closed handling breaks. Re-establishing that injection at
the new coordinator boundary is in scope for #3052 and is not verified by any
contract in this file.
"""

from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import threading
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

_spec = importlib.util.spec_from_file_location(
    "tape_ec_harness", Path(__file__).with_name("harness.py")
)
assert _spec and _spec.loader
harness = importlib.util.module_from_spec(_spec)
# Register before exec: @dataclass resolves its annotations through
# sys.modules[cls.__module__], which is None for a module loaded by path alone.
sys.modules[_spec.name] = harness
_spec.loader.exec_module(harness)
ContractFailure = harness.ContractFailure

CASE_ID = "ec-3052-durability"
TOPIC = "ec-3052-durability"
WRITERS = 8
EVENTS = 400
# Kill once this many appends have been acknowledged. Far enough in that the
# server is in steady state and any batching window is full, far enough from
# EVENTS that writers are still in flight when the process dies -- a kill after
# the last writer finished would test a clean shutdown, which is not the claim.
KILL_AFTER_ACKS = 120
# Deliberately-malformed appends submitted before the writers start, to give
# the `recovered ∩ refused = ∅` assertion something to assert on. Small: they
# are a probe, not a load. Their bodies are truncated JSON that still contains
# the key, so recovering one would mean the server stored a request it refused.
REFUSAL_PROBES = 12


def malformed_body(key: str) -> bytes:
    """A body that cannot parse but does carry `key` in its bytes.

    Truncated mid-object rather than, say, `b"{"`: the point of the probe is
    that the key was visibly available to anything willing to read past the
    syntax error, so a recovered probe key is unambiguous evidence of a stored
    refusal rather than a coincidence.
    """
    return f'{{"key": "{key}", "payload": {{"n": 1'.encode()


def check_kill_preconditions(
    reached_threshold: bool, acked: int, probe_refused: int, wrongly_refused: int
) -> None:
    """Refuse to grade a run that never put the property at risk.

    Two directions of vacuity, not one. A kill that landed before anything was
    durable proves nothing about durability; a run in which the refusal probes
    were *not* refused proves nothing about resurrection, because the assertion
    reading that set would have no members to reject. Both report green if left
    ungraded. This is a separate function because the version of this guard that
    lived inline was dead code: the kill thread fired unconditionally after its
    deadline, so the flag it was reading was always set. Dead guards are
    invisible; unit-tested ones are not.
    """
    if not reached_threshold:
        raise ContractFailure(
            f"the kill fired on a timeout, not on the acknowledgement "
            f"threshold: only {acked} appends were acknowledged, below the "
            f"{KILL_AFTER_ACKS} required. The run proves nothing about "
            f"durability."
        )
    # Unreachable while the flag above is set from the same counter this reads,
    # and kept anyway: it is the invariant the flag is *supposed* to encode, and
    # a future refactor that sets the flag from somewhere else would otherwise
    # lower the bar in silence rather than fail here.
    if acked < KILL_AFTER_ACKS:
        raise ContractFailure(
            f"only {acked} appends were acknowledged, below the "
            f"{KILL_AFTER_ACKS} threshold the kill was supposed to wait for"
        )
    if wrongly_refused:
        raise ContractFailure(
            f"{wrongly_refused} of {EVENTS} well-formed appends were refused "
            f"with a non-2xx status by a healthy server before the kill. "
            f"Throughput of failures is not durability; fix the rejections "
            f"before grading recovery."
        )
    if probe_refused != REFUSAL_PROBES:
        raise ContractFailure(
            f"only {probe_refused} of {REFUSAL_PROBES} malformed appends were "
            f"refused; the rest were accepted or went unanswered. The "
            f"'a refused write must not come back' assertion has nothing to "
            f"assert on, so a green verdict here would be vacuous."
        )


def check_recovery(
    sent: set[str],
    acked: dict[str, int],
    refused: set[str],
    events: list[dict],
) -> dict:
    """The contract itself, as a pure function over four observations.

    Kept separate from the process orchestration so it can be tested against
    synthetic inputs -- including each failure it is supposed to catch. An
    oracle that has never been shown to go red is not known to be an oracle;
    the assertions below are the entire value of this contract, and they are
    also the easiest thing to get subtly wrong.
    """
    keys = [event.get("key") for event in events]
    offsets = [int(event["offset"]) for event in events]

    # First, because a log applied twice on recovery is *the* canonical WAL
    # defect and every check below is blind to it: the by-key index built next
    # would silently collapse the duplicates, and the offsets would still come
    # out distinct and contiguous because the server assigns them fresh on each
    # apply.
    duplicated = sorted({key for key in keys if keys.count(key) > 1})
    if duplicated:
        raise ContractFailure(
            f"replay returned {len(keys)} events for {len(set(keys))} distinct "
            f"keys; {len(duplicated)} key(s) came back more than once "
            f"(first: {duplicated[:5]}). The log was applied more than once on "
            f"recovery."
        )

    recovered: dict[str, dict] = {event["key"]: event for event in events}

    lost = sorted(set(acked) - set(recovered))
    if lost:
        raise ContractFailure(
            f"{len(lost)} acknowledged appends did not survive SIGKILL "
            f"(first: {lost[:5]}). RPO is not 0; the durability barrier "
            f"returned before the data was durable."
        )

    resurrected = sorted(set(recovered) & refused)
    if resurrected:
        raise ContractFailure(
            f"{len(resurrected)} appends the server explicitly refused with a "
            f"non-2xx status came back after restart (first: "
            f"{resurrected[:5]}). A declined write that becomes durable anyway "
            f"is a lie told to the client."
        )

    phantom = sorted(set(recovered) - sent)
    if phantom:
        raise ContractFailure(
            f"{len(phantom)} recovered records were never submitted by this "
            f"client (first: {phantom[:5]}). Recovery is inventing data."
        )

    if len(set(offsets)) != len(offsets):
        raise ContractFailure(
            f"replay returned duplicate offsets: {len(offsets)} events, "
            f"{len(set(offsets))} distinct offsets"
        )
    if offsets and sorted(offsets) != list(range(min(offsets), max(offsets) + 1)):
        raise ContractFailure(
            f"recovered offsets are not contiguous between {min(offsets)} "
            f"and {max(offsets)}; replay has a hole"
        )

    corrupt = [
        key
        for key, n in acked.items()
        if recovered[key].get("payload", {}).get("n") != n
    ]
    if corrupt:
        raise ContractFailure(
            f"{len(corrupt)} recovered payloads do not match what was "
            f"acknowledged (first: {corrupt[:5]})"
        )

    return {
        "submitted": len(sent),
        "acknowledged": len(acked),
        "refused": len(refused),
        "recovered": len(recovered),
        "acknowledged_and_recovered": len(acked),
        "recovered_beyond_acknowledged": len(set(recovered) - set(acked)),
        "offset_range": [min(offsets), max(offsets)] if offsets else [],
    }


def verify() -> dict:
    binary = harness.build_binary()
    with tempfile.TemporaryDirectory(prefix="ec-3052-durability-") as tmp:
        workdir = Path(tmp)
        data_dir = workdir / "data"
        data_dir.mkdir()
        server = harness.start_server(
            binary, data_dir, workdir / "server-1.log"
        )

        lock = threading.Lock()
        acked: dict[str, int] = {}
        sent: set[str] = set()
        refused: set[str] = set()
        unknown: set[str] = set()
        outcome: dict[str, object] = {"reached_threshold": False}

        # The refusal probes go first, serially, against a server that is
        # healthy and not yet under load. Interleaving them with the writers
        # would leave whether they were answered before the kill up to timing,
        # and a precondition that has to tolerate "some probes went unanswered"
        # is a weaker precondition. Here all of them are answered or the run is
        # graded ungradeable, which is the correct outcome either way.
        probes: set[str] = set()
        probe_refused = 0
        for n in range(REFUSAL_PROBES):
            key = f"m{n:06d}"
            probes.add(key)
            sent.add(key)
            if not 200 <= harness.append_raw(
                server.base, TOPIC, malformed_body(key)
            ) < 300:
                refused.add(key)
                probe_refused += 1

        def submit(n: int) -> None:
            key = f"k{n:06d}"
            with lock:
                sent.add(key)
            try:
                status = harness.append(server.base, TOPIC, key, {"n": n})
            except Exception:
                # No status at all: the socket died under us. The kill lands
                # mid-flight for some writers and their outcome is genuinely
                # unknown to the client, which is the honest state to record --
                # not a failure, and not an ack. This branch stays narrow only
                # because `harness.append` returns non-2xx codes instead of
                # raising them; see its docstring.
                with lock:
                    unknown.add(key)
                return
            if 200 <= status < 300:
                with lock:
                    acked[key] = n
            else:
                # A status came back, so the server was alive and answered. It
                # declined this write, which is a promise in the other
                # direction and must be held to.
                with lock:
                    refused.add(key)

        def killer() -> None:
            deadline = time.monotonic() + 120
            while time.monotonic() < deadline:
                with lock:
                    if len(acked) >= KILL_AFTER_ACKS:
                        outcome["reached_threshold"] = True
                        break
                time.sleep(0.01)
            # Kill either way: the writer pool would otherwise block forever
            # waiting on a server nobody is going to stop. Whether the threshold
            # was actually reached is recorded above and graded afterwards, so a
            # timeout can never masquerade as a successful run.
            server.kill9()

        kill_thread = threading.Thread(target=killer, daemon=True)
        kill_thread.start()
        with ThreadPoolExecutor(max_workers=WRITERS) as pool:
            list(pool.map(submit, range(EVENTS)))
        kill_thread.join(timeout=150)

        if server.process.returncode is None:
            server.terminate()
            raise ContractFailure("server did not actually die under SIGKILL")
        check_kill_preconditions(
            bool(outcome["reached_threshold"]),
            len(acked),
            probe_refused,
            len(refused - probes),
        )

        # Restart from the same data-dir. Anything the first process made
        # durable has to come back on its own; nothing is repaired by hand.
        restarted = harness.start_server(
            binary, data_dir, workdir / "server-2.log"
        )
        try:
            events = harness.replay_all(restarted.base, TOPIC)
        finally:
            restarted.terminate()

        facts = check_recovery(sent, acked, refused, events)
        facts["in_flight_at_kill_outcome_unknown"] = len(unknown)
        facts["refusals_probed"] = probe_refused
        return facts


def main() -> int:
    try:
        facts = verify()
    except harness.ContractFailure as failure:
        harness.write_evidence(
            CASE_ID, "failed", {"error": str(failure), **failure.facts}
        )
        print(f"FAIL {CASE_ID}: {failure}", file=sys.stderr)
        return 1
    path = harness.write_evidence(CASE_ID, "passed", facts)
    print(f"PASS {CASE_ID}: {json.dumps(facts, sort_keys=True)}")
    print(f"evidence: {path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
