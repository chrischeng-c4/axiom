# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "identity"
# lib = "hashlib"
# dimension = "concurrency"
# case = "parent_hash_object_hexdigests_in_child"
# subject = "hashlib.sha256"
# kind = "semantic"
# xfail = "mamba keys hashlib state by a u64 index into a thread-local registry; a hash object CREATED IN THE PARENT misses the child's registry and .hexdigest resolves to None, giving 'NoneType' object is not callable (#2968). A hash object created inside the child works, so the defect is the parent-to-child handoff, not the module."
# mem_carveout = ""
# source = "#2968 stage-1 ownership probe matrix"
# status = "filled"
# ///
"""Concurrency contract: a stdlib handle outlives the thread that constructed it.

The hash object is built on the main thread and digested from a worker. That
parent-to-child handoff is the whole point: an equivalent object constructed
inside the worker already works in mamba today, so a fixture that builds it in
the child would pass and prove nothing.

sha256 of b"abc" is a published vector, so this fixture carries its own oracle
and does not depend on the reference interpreter agreeing with anything.
Deterministic -- not a race.

The verdict is the digest value, never the exit status. An uncaught exception in
a non-main thread leaves the process exit status at 0 in CPython too, so exit
status cannot witness a thread-internal failure in either runtime.
"""
import hashlib
import threading

EXPECTED = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"

digest = hashlib.sha256(b"abc")

result: list = []
error: list = []


def worker() -> None:
    try:
        result.append(digest.hexdigest())
    except BaseException as exc:
        error.append(f"{type(exc).__name__}: {exc}")


t = threading.Thread(target=worker)
t.start()
t.join()

if error:
    print(f"concurrency: FAIL: parent hash object unusable in child: {error[0]}")
elif not result:
    print("concurrency: FAIL: worker produced no digest and raised nothing")
elif result[0] != EXPECTED:
    print(f"concurrency: FAIL: wrong digest in child: {result[0]} expected={EXPECTED}")
else:
    print("concurrency: PASS")
