# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "primitives"
# lib = "threading"
# dimension = "concurrency"
# case = "get_ident_distinct_for_live_threads"
# subject = "threading.get_ident"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "concurrency_matrix.py (thread identity primitive)"
# status = "filled"
# ///
"""Concurrency primitive: simultaneously-live threads have distinct identities.

N threads are held alive at the same instant, then each records
threading.get_ident(). The contract: N concurrently-live threads yield N
distinct ids. CPython (GIL and free-threaded alike) gives exactly N, and so
does mamba.

The threads are held overlapping with a sleep rather than a Barrier ON PURPOSE.
An earlier revision rendezvoused on `threading.Barrier(N)` and reported
`FAIL: 0 distinct ids for 6 live threads`, which reads like a get_ident defect
and was documented as one ("mamba returns one shared id for all threads").
That diagnosis was wrong. Zero distinct ids meant the list was *empty* -- no
worker ever reached the append, because mamba's `Barrier.wait()` silently drops
every waiter (measured: 0 of 6 get past it, no exception, exit 0 -- see #2029).
get_ident itself is fine: without the Barrier, mamba returns 6 distinct ids for
6 overlapping threads, same as CPython.

So do not reintroduce a Barrier here. This case gates thread identity; the
Barrier gap is gated by barrier_rounds_complete_without_deadlock.py, and mixing
them makes a broken Barrier masquerade as a broken get_ident.

A sleep is a weaker liveness guarantee than a rendezvous, but it is the strongest
one available that does not depend on a primitive this fixture is not testing.
Ids only have to be distinct among *concurrently live* threads -- CPython
explicitly recycles an id once a thread exits -- so the overlap is what matters.
"""
import threading
import time

N = 6
ids: list[int] = []
collect = threading.Lock()


def worker() -> None:
    # Hold every worker alive past the point where the last one starts, so all
    # N are live at the same instant and no id can be recycled.
    time.sleep(0.05)
    me = threading.get_ident()
    with collect:
        ids.append(me)


threads = [threading.Thread(target=worker) for _ in range(N)]
for t in threads:
    t.start()
for t in threads:
    t.join()

if len(ids) != N:
    print(f"concurrency: FAIL: {len(ids)} of {N} workers recorded an id")
else:
    distinct = len(set(ids))
    if distinct == N:
        print("concurrency: PASS")
    else:
        print(f"concurrency: FAIL: {distinct} distinct ids for {N} live threads")
