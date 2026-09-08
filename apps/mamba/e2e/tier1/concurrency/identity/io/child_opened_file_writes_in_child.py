# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "identity"
# lib = "io"
# dimension = "concurrency"
# case = "child_opened_file_writes_in_child"
# subject = "open"
# kind = "self-verdict"
# mem_carveout = ""
# source = "#2968 stage-1 ownership probe matrix"
# status = "filled"
# ///
"""Intended as the control for `parent_file_handle_writes_in_child` -- but red.

The plan was: open and write entirely inside the worker, expect green, and
thereby prove the sibling xfail indicts the thread boundary rather than file
I/O. Instead this surfaced a second, independent defect. `with open(...) as fh:`
raises `UnboundLocalError` on `fh` when the block runs on a worker thread, while

  * the identical `with` form works on the main thread (module scope and inside
    a plain function), and
  * explicit `fh = open(...)` / `write` / `close` works inside the same worker.

So the write path is fine in a child; the context-manager `as` binding is not.
Tracked as #3101. Once that lands this fixture reverts to its control role and
the xfail is removed.
"""
import os
import tempfile
import threading

directory = tempfile.mkdtemp()
path = os.path.join(directory, "written_by_child.txt")

error: list = []


def worker() -> None:
    try:
        with open(path, "w") as fh:
            fh.write("payload")
    except BaseException as exc:
        error.append(f"{type(exc).__name__}: {exc}")


t = threading.Thread(target=worker)
t.start()
t.join()

contents = ""
if os.path.exists(path):
    with open(path) as fh:
        contents = fh.read()
    os.remove(path)
os.rmdir(directory)

if error:
    print(f"concurrency: FAIL: child-opened file not writable: {error[0]}")
elif contents != "payload":
    print(f"concurrency: FAIL: child write lost: file holds {contents!r} expected 'payload'")
else:
    print("concurrency: PASS")
