# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "real_world"
# case = "transactional_resource_cleanup"
# subject = "contextlib.ExitStack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.ExitStack: a transactional batch job acquires several resources through one ExitStack, commits on success, and on a mid-batch failure unwinds every already-acquired resource in LIFO order before the error surfaces — the canonical all-or-nothing cleanup pattern"""
import contextlib

events: list = []


class Resource:
    """A pretend connection/handle that logs acquire and release."""

    def __init__(self, name):
        self.name = name

    def __enter__(self):
        events.append(f"acquire:{self.name}")
        return self

    def __exit__(self, exc_type, exc, tb):
        events.append(f"release:{self.name}")
        return False  # never suppress; this is cleanup, not error-handling


def run_batch(rows):
    """Process every row under one ExitStack. Any failure rolls everything
    back: already-acquired resources are released LIFO before the error
    surfaces, so a half-applied batch is never left open."""
    processed = []
    with contextlib.ExitStack() as stack:
        for row in rows:
            res = stack.enter_context(Resource(f"row{row}"))
            if row == "bad":
                raise ValueError(f"cannot process {res.name}")
            processed.append(res.name)
    return processed


# 1. Happy path: all resources acquired, all released LIFO, work committed.
events.clear()
done = run_batch([0, 1, 2])
assert done == ["row0", "row1", "row2"], done
assert events == [
    "acquire:row0", "acquire:row1", "acquire:row2",
    "release:row2", "release:row1", "release:row0",
], events

# 2. Failure path: row "bad" fails after row0/row1 are open; ExitStack unwinds
#    every acquired resource in LIFO order, then the error propagates.
events.clear()
failed = False
try:
    run_batch([0, 1, "bad", 3])
except ValueError as e:
    failed = True
    assert "row" in str(e), str(e)
assert failed, "the batch failure must surface to the caller"
assert events == [
    "acquire:row0", "acquire:row1", "acquire:rowbad",
    "release:rowbad", "release:row1", "release:row0",
], events

print("transactional_resource_cleanup OK")
