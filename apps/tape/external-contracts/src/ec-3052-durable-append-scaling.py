"""EC ec-3052-scaling: durable append throughput rises with concurrency.

The oracle is a ratio measured by an external client, not a number read out of
the server. Two independent servers, each with its own empty `--data-dir`, are
driven at 1 and at 16 concurrent connections, and the contract is that the
16-connection run commits meaningfully more events per second than the
1-connection run.

### Why a ratio and not a number

Absolute throughput here would be a capacity claim on a developer laptop, which
#3052 rules out until the measurement is taken on target hardware. Worse, the
absolutes do not transfer: macOS `F_FULLFSYNC` is a real drive-cache flush and
a GKE PD `fsync()` is a different primitive with different costs. What does
transfer is the shape -- barrier cost is nearly flat in batch size, so
amortising a barrier across N concurrent writers has to show up as throughput
rising with N. A ratio states that shape and stays true on both platforms.

### Why a bare ratio is not enough, in both directions

A ratio alone is satisfiable by making the *baseline worse*. Group commit with
a fixed linger window and no early flush does exactly that: a lone writer waits
out the full window for batch partners who never arrive, so 1 connection drops
to a fraction of today's rate while 16 rises, and the ratio sails through while
single-writer durable append has regressed several-fold. So the contract puts a
**floor** under the 1-connection level.

It is also satisfiable by removing the durability instead of amortising it. A
build that acknowledges before its barrier -- or one that quietly drops
`FsyncPolicy::Always` to `EverySec`, which #3052's Out of Scope explicitly
forbids because it trades away RPO 0 -- posts a superb ratio and a superb
baseline. The recovery check in `measure()` does not catch this: SIGKILL kills
a process, not the page cache, so unsynced writes are still readable by the
next process (see `harness.py`). Every event would come back. So the contract
also puts a **ceiling** over both levels.

Both bounds are expressed against a barrier cost measured *on the same host in
the same run*, which is what makes them hardware-relative and therefore
transferable, exactly like the ratio. The barrier is measured with the
primitive Rust's `sync_all` actually uses on each platform -- `F_FULLFSYNC` on
macOS, `fsync()` elsewhere. That distinction is not pedantry: on this host
Python's bare `os.fsync` measures 0.032 ms and `F_FULLFSYNC` measures 5.07 ms,
a factor of 158, and calibrating against the wrong one would set bounds no
implementation could ever fail.

The ceiling is the arithmetic of closed-loop clients: with N connections at
most N records can be in flight, so at most N records can ride one barrier, so
`ops_per_s <= N * barrier_hz`. `BARRIER_COST_CEILING` multiplies that by 3 to
absorb the probe being pessimistic about the product's barrier -- `sync_data`
is `fdatasync` on Linux and can beat the probe's `fsync` on a preallocated
file.

That it fires on a real barrier-eliding build is measured, not argued. `tape
serve` with neither `--store` nor `--data-dir` leaves `AppState.store` as
`None`, so `persist` returns `Ok(())` without ever issuing a barrier
(`apps/tape/src/server.rs:188-190`) -- a genuine zero-durability tape needing
no patched source. Driven by this same harness:

    barrier 198.1/s   1 conn   4786.8 ops/s   ceiling  594.3   FIRES  (8.05x)
                     16 conn   4785.9 ops/s   ceiling 9508.8   silent (0.50x)

So the discriminating power lives at the **1-connection** level, and it is not
marginal: 8x over, against a bound with 3x of slack already in it. The
16-connection ceiling is silent here, and the reason is worth stating rather
than leaving a reader to assume both levels are guarded -- that build is HTTP
bound at ~4786 ops/s at every concurrency, which is below `16 * 3 *
barrier_hz`. It is retained because it does bind the case this host cannot
exhibit: a build that elides the barrier *and* parallelises, whose scaled level
would run past 9508 while its baseline stayed plausible.

Incidentally that zero-durability build also fails the ratio, at 1.00 -- tape's
global append mutex keeps it flat with or without a barrier. That is luck, not
coverage, and is exactly why the ceiling is a separate check: the next build to
remove the barrier may well be one that scales.

### Why each level is killed and restarted before it is counted

Counting a level's events against the server that just wrote them proves
nothing at all: `TapeJournal::replay_refs` (`apps/tape/src/lib.rs:203-213`)
serves from the in-memory `topics` map, so a build that lost every byte on
disk would still replay the full set back. Killing first at least forces the
count to come out of a file.

It does not do more than that, and this contract does not claim it does.
`survived_restart` establishes that the writes left the process, not that they
reached the medium; the page cache survives SIGKILL. The ceiling above is what
covers the remaining gap, and it covers it by arithmetic rather than by
observation: a build that never waits for a barrier cannot also be running at
barrier speed.

### Why the two levels get separate servers

The path being replaced rewrites the *entire* journal on every mutation, so
each append costs more than the one before it. Running both levels against one
server would charge the second level for the first level's accumulated bytes.
Whichever level ran second would look worse for a reason that has nothing to do
with concurrency, and on the current code the second level is the one this
contract is trying to observe.

### Expected verdict before #3052 lands

This contract **fails on the current code, by construction**, and was observed
doing so before a line of #3052 was written. Five runs against unchanged HEAD,
one of them on an independent reviewer's host:

    barrier    1 conn    16 conn    ratio    fraction
    197.0/s    61.0       83.5      1.37     0.310
    (n/a)      78.9       82.6      1.05     --      (predates the barrier probe)
    184.6/s    73.3       70.2      0.96     0.397
    167.3/s    65.8       68.7      1.04     0.393   (independent host)
    195.7/s    85.5       93.8      1.10     0.437
    192.6/s    63.0       87.6      1.39     0.327

The absolute numbers move with host load, which is exactly why the contract is
a ratio; the ratio stays pinned near 1.0 in every run, against a required 4.0.
Every level reports `survived_restart: 620`.

The fraction column is the whole-file path's own cost -- 0.310 to 0.437, i.e.
about 2.3 to 3.2 barriers per append, which is what `atomic_write`'s two
barriers plus a full rewrite buy -- and it is what the floor is derived from.
`LONE_WRITER_BARRIER_FRACTION` is 0.25, four barriers per append: strictly
below every observation of the code being replaced. That is what "no
regression" has to mean if the floor is to be a check at all rather than a
second improvement requirement hiding behind the ratio.

The first choice was 0.4, and the spread above is why it was wrong. It falls
*inside* the incumbent's own range: red on four of these five runs and green on
the fifth, on identical code. A threshold that a fixed binary crosses back and
forth is not a bound on that binary's behaviour, it is a sampling artefact --
and it stayed invisible because the ratio check fires first and no run ever
reached it.

Neither red is a defect in the contract; a verifier for a capability that does
not exist yet is supposed to be red. Both turn green when the WAL and group
commit land, and their value afterwards is that they stay green -- they are
what would catch someone quietly reintroducing a per-request barrier, or
buying the ratio by starving the single writer.
"""

from __future__ import annotations

import importlib.util
import json
import os
import statistics
import sys
import tempfile
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

CASE_ID = "ec-3052-scaling"
BASELINE_CONNECTIONS = 1
SCALED_CONNECTIONS = 16
# Enough events that startup and the first barrier are not a material share of
# the window, few enough that the serial level still finishes: at the measured
# 61-89 ops/s the 1-connection level takes about 7-10s.
EVENTS = 600
WARMUP = 20
# Group commit's own arithmetic says a 16-way batch amortises a barrier that is
# nearly flat in batch size, so the ceiling is far above this (#3052 measures
# 8 records/barrier at 1452/s against 177/s for one). 4.0 is set well under
# that on purpose -- the contract is "throughput rises with concurrency", and a
# threshold tuned to the measured ceiling would turn every unrelated scheduling
# hiccup into a false red. It is also set well above the ~2.0 that WAL-without-
# group-commit buys, which is the improvement that must NOT be enough.
REQUIRED_RATIO = 4.0
# Records per barrier for a lone writer, as a lower bound. A lone durable
# append should cost about one barrier once the WAL lands, so the post-change
# fraction should be near 1.0 and this floor is 4x clear of it. It is set at
# 0.25 -- four barriers per append -- because that is strictly below the
# incumbent's own measured cost in every observation (0.310 through 0.437; see
# the docstring table), which makes it the no-regression bound it is documented
# to be. 0.4 was tried first and was wrong: it falls inside that spread, so the
# code it claimed to be pinned to was red on four runs and green on one, and
# the flapping stayed invisible because the ratio check fires first and no run
# ever reached it. Meanwhile the defect it exists to
# catch -- fixed linger with no early flush, ~20 ops/s against a 197/s barrier,
# fraction 0.10 -- is still 2.5x below 0.25.
LONE_WRITER_BARRIER_FRACTION = 0.25
# Records per barrier per in-flight connection, as an upper bound. With N
# closed-loop connections at most N records are in flight, so at most N can
# share one barrier: ops_per_s <= N * barrier_hz is arithmetic, not a tuning
# choice. The 3x slack absorbs the probe measuring a costlier primitive than
# the product's (`sync_data` is `fdatasync` on Linux). What it does not absorb
# is a build that skipped the barrier: measured at 8.05x over the bound at 1
# connection against a real zero-durability tape (see the docstring table).
# This is the only check here that can distinguish "amortised the barrier" from
# "removed it", because SIGKILL cannot -- see the docstring.
BARRIER_COST_CEILING = 3.0
BARRIER_SAMPLES = 40


def measure_barrier(workdir: Path) -> dict:
    """Median cost of the durability barrier Rust would use on this platform.

    `os.fsync` is the wrong probe on macOS: CPython issues a plain `fsync()`,
    which returns once the write reaches the drive's cache, while Rust's
    `sync_all` issues `F_FULLFSYNC`, which waits for the cache to be flushed.
    Measuring the cheap one would calibrate the floor against a barrier the
    product never pays.
    """
    if sys.platform == "darwin":
        import fcntl

        f_fullfsync = getattr(fcntl, "F_FULLFSYNC", 51)

        def barrier(fd: int) -> None:
            fcntl.fcntl(fd, f_fullfsync)

        primitive = "F_FULLFSYNC"
    else:
        barrier = os.fsync
        primitive = "fsync"

    path = workdir / "barrier-probe"
    fd = os.open(path, os.O_CREAT | os.O_WRONLY, 0o600)
    try:
        os.write(fd, b"x" * 512)
        barrier(fd)  # first one is cold; not sampled
        samples = []
        for _ in range(BARRIER_SAMPLES):
            os.write(fd, b"y" * 512)
            started = time.monotonic()
            barrier(fd)
            samples.append(time.monotonic() - started)
    finally:
        os.close(fd)
    median = statistics.median(samples)
    if median <= 0:
        raise ContractFailure(
            f"the {primitive} probe measured a non-positive median "
            f"({median}); the barrier floor cannot be calibrated"
        )
    return {
        "primitive": primitive,
        "samples": BARRIER_SAMPLES,
        "median_ms": round(median * 1000, 3),
        "barrier_hz": round(1.0 / median, 1),
    }


def measure(binary: Path, workdir: Path, connections: int) -> dict:
    """Drive one fresh server at a fixed concurrency and return its rate."""
    data_dir = workdir / f"data-{connections}"
    data_dir.mkdir()
    topic = f"ec-3052-scaling-{connections}"
    server = harness.start_server(
        binary, data_dir, workdir / f"server-{connections}.log"
    )
    killed = False
    try:
        # Warm up outside the measured window: the first append pays for topic
        # creation and the first file materialisation, neither of which is the
        # steady-state cost being compared.
        # The status is checked here rather than discarded. Before `append`
        # was taught to return non-2xx codes instead of raising them, a refused
        # warm-up aborted the run loudly; afterwards it would have continued in
        # silence and the measured window would have run against a server
        # already refusing writes -- which measures the rate of saying no.
        for n in range(WARMUP):
            status = harness.append(server.base, topic, f"w{n:06d}", {"n": n})
            if not 200 <= status < 300:
                raise ContractFailure(
                    f"warm-up append {n} was refused with HTTP {status} at "
                    f"{connections} connection(s); the server was not healthy "
                    f"before the measured window opened"
                )

        failures: list[str] = []

        def submit(n: int) -> None:
            try:
                status = harness.append(server.base, topic, f"k{n:06d}", {"n": n})
            except Exception as error:  # noqa: BLE001 - reported, not swallowed
                failures.append(f"{n}: {error}")
                return
            if not 200 <= status < 300:
                failures.append(f"{n}: HTTP {status}")

        started = time.monotonic()
        with ThreadPoolExecutor(max_workers=connections) as pool:
            list(pool.map(submit, range(EVENTS)))
        elapsed = time.monotonic() - started

        if failures:
            raise ContractFailure(
                f"{len(failures)} of {EVENTS} appends failed at {connections} "
                f"connections (first: {failures[:3]}). Throughput is not "
                f"comparable when writes are being rejected."
            )

        # SIGKILL, then count from a fresh process reading the same data
        # directory. Asking the still-running server would only prove the
        # events reached its in-memory map, which an ack-before-barrier build
        # would satisfy while scoring beautifully.
        server.kill9()
        killed = True
    finally:
        if not killed:
            server.terminate()

    restarted = harness.start_server(
        binary, data_dir, workdir / f"server-{connections}-restarted.log"
    )
    try:
        events = harness.replay_all(restarted.base, topic)
    finally:
        restarted.terminate()

    if len(events) != EVENTS + WARMUP:
        raise ContractFailure(
            f"at {connections} connections the server acknowledged "
            f"{EVENTS + WARMUP} appends but only {len(events)} survived a "
            f"SIGKILL and restart. Throughput that is not durable does not "
            f"count."
        )

    return {
        "connections": connections,
        "events": EVENTS,
        "elapsed_s": round(elapsed, 3),
        "ops_per_s": round(EVENTS / elapsed, 1),
        "survived_restart": len(events),
    }


def check_scaling(baseline: dict, scaled: dict, barrier: dict) -> dict:
    """The contract itself, as a pure function over three measurements.

    Separated from the measurement so the threshold logic can be tested
    without starting a server -- including the degenerate inputs (a zero or
    negative baseline rate) where a naive ratio would divide by zero or,
    worse, come out large enough to pass.
    """
    base_rate = baseline["ops_per_s"]
    scaled_rate = scaled["ops_per_s"]
    if base_rate <= 0 or scaled_rate <= 0:
        raise ContractFailure(
            f"a measured rate was not positive (baseline {base_rate} ops/s, "
            f"scaled {scaled_rate} ops/s); there is no ratio to judge"
        )
    barrier_hz = barrier["barrier_hz"]
    if barrier_hz <= 0:
        raise ContractFailure(
            f"the measured barrier rate was not positive ({barrier_hz}/s); "
            f"the lone-writer floor cannot be judged"
        )

    ratio = scaled_rate / base_rate
    floor = LONE_WRITER_BARRIER_FRACTION * barrier_hz
    facts = {
        "baseline": baseline,
        "scaled": scaled,
        "barrier": barrier,
        "ratio": round(ratio, 2),
        "required_ratio": REQUIRED_RATIO,
        "lone_writer_floor_ops_per_s": round(floor, 1),
        "lone_writer_barrier_fraction": round(base_rate / barrier_hz, 3),
        "barrier_cost_ceiling": BARRIER_COST_CEILING,
        "ceiling_ops_per_s": {
            str(level["connections"]): round(
                BARRIER_COST_CEILING * level["connections"] * barrier_hz, 1
            )
            for level in (baseline, scaled)
        },
    }

    # Before the ratio, deliberately: a build that eliminated the barrier
    # rather than amortising it posts an excellent ratio and would sail past a
    # check placed after it.
    for level in (baseline, scaled):
        connections = level["connections"]
        ceiling = BARRIER_COST_CEILING * connections * barrier_hz
        if level["ops_per_s"] > ceiling:
            raise ContractFailure(
                f"{level['ops_per_s']} ops/s at {connections} connection(s) is "
                f"{level['ops_per_s'] / (connections * barrier_hz):.1f} records "
                f"per barrier per in-flight writer, against a measured "
                f"{barrier_hz}/s {barrier['primitive']} barrier. At most "
                f"{connections} record(s) can be in flight and therefore share "
                f"one barrier, so anything above {BARRIER_COST_CEILING}x that "
                f"({ceiling:.1f} ops/s) means the acknowledgement did not wait "
                f"for a barrier at all. This is faster than durable, not "
                f"durable and fast.",
                facts,
            )

    if ratio < REQUIRED_RATIO:
        raise ContractFailure(
            f"durable append throughput did not rise with concurrency: "
            f"{scaled_rate} ops/s at {scaled['connections']} connections "
            f"vs {base_rate} ops/s at {baseline['connections']} "
            f"(ratio {ratio:.2f}, required {REQUIRED_RATIO}). A flat line here "
            f"means every append is still paying for its own durability "
            f"barrier instead of sharing one.",
            facts,
        )
    if base_rate < floor:
        raise ContractFailure(
            f"the ratio was bought by starving the lone writer: "
            f"{base_rate} ops/s at {baseline['connections']} connection "
            f"against a measured {barrier_hz}/s "
            f"{barrier['primitive']} barrier is "
            f"{base_rate / barrier_hz:.2f} of one barrier's worth of work, "
            f"below the required {LONE_WRITER_BARRIER_FRACTION}. A single "
            f"writer must flush its own batch immediately instead of waiting "
            f"out a linger window for partners who are not coming.",
            facts,
        )
    return facts


def verify() -> dict:
    binary = harness.build_binary(release=True)
    with tempfile.TemporaryDirectory(prefix="ec-3052-scaling-") as tmp:
        workdir = Path(tmp)
        barrier = measure_barrier(workdir)
        baseline = measure(binary, workdir, BASELINE_CONNECTIONS)
        scaled = measure(binary, workdir, SCALED_CONNECTIONS)
    return check_scaling(baseline, scaled, barrier)


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
