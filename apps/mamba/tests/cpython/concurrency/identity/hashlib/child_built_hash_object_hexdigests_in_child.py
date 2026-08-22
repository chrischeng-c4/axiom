# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "identity"
# lib = "hashlib"
# dimension = "concurrency"
# case = "child_built_hash_object_hexdigests_in_child"
# subject = "hashlib.sha256"
# kind = "semantic"
# mem_carveout = ""
# source = "#2968 stage-1 ownership probe matrix"
# status = "filled"
# ///
"""Control for `parent_hash_object_hexdigests_in_child`: same call, child-built.

This one passes in mamba today and is expected to keep passing. It exists to
localize the sibling xfail: with both present, a red pair means "hashlib is
broken under threads" while a red parent-case and a green child-case means
"the parent-to-child handoff is broken". Without this control the xfail alone
cannot distinguish the two, and the first reading of the probe evidence drew
exactly that wrong conclusion.

It is also the regression guard for the fix: whatever makes the parent case work
must not break construction inside the worker.
"""
import hashlib
import threading

EXPECTED = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"

result: list = []
error: list = []


def worker() -> None:
    try:
        result.append(hashlib.sha256(b"abc").hexdigest())
    except BaseException as exc:
        error.append(f"{type(exc).__name__}: {exc}")


t = threading.Thread(target=worker)
t.start()
t.join()

if error:
    print(f"concurrency: FAIL: child-built hash object unusable: {error[0]}")
elif not result:
    print("concurrency: FAIL: worker produced no digest and raised nothing")
elif result[0] != EXPECTED:
    print(f"concurrency: FAIL: wrong digest in child: {result[0]} expected={EXPECTED}")
else:
    print("concurrency: PASS")
