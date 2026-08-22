# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "identity"
# lib = "io"
# dimension = "concurrency"
# case = "parent_file_handle_writes_in_child"
# subject = "open"
# kind = "semantic"
# xfail = "mamba represents a file object as a u64 index into a thread-local registry; a handle OPENED IN THE PARENT misses the child's registry and degrades to a bare int, raising 'int' object has no attribute 'write' (#2968). A file opened inside the child works, so the defect is the parent-to-child handoff."
# mem_carveout = ""
# source = "#2968 stage-1 ownership probe matrix"
# status = "filled"
# ///
"""Concurrency contract: a file opened on one thread is writable from another.

Paired with `child_opened_file_writes_in_child.py` to separate "file I/O is
broken under threads" from "the handoff is broken".

This case is the most consequential of the handle family, because the failure is
a lost write: a program that logs from worker threads through a handle opened at
startup produces an empty file and no error. The verdict is therefore the
readback of the file contents, not merely the absence of an exception.

Deterministic, single writer, no interleaving question.
"""
import os
import tempfile
import threading

directory = tempfile.mkdtemp()
path = os.path.join(directory, "written_by_child.txt")
handle = open(path, "w")

error: list = []


def worker() -> None:
    try:
        handle.write("payload")
    except BaseException as exc:
        error.append(f"{type(exc).__name__}: {exc}")


t = threading.Thread(target=worker)
t.start()
t.join()
handle.close()

with open(path) as fh:
    contents = fh.read()

os.remove(path)
os.rmdir(directory)

if error:
    print(f"concurrency: FAIL: parent file handle not writable in child: {error[0]}")
elif contents != "payload":
    print(f"concurrency: FAIL: child write lost: file holds {contents!r} expected 'payload'")
else:
    print("concurrency: PASS")
