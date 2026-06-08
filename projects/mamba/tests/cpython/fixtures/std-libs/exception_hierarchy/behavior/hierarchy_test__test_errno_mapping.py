# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_hierarchy"
# dimension = "behavior"
# case = "hierarchy_test__test_errno_mapping"
# subject = "cpython.test_exception_hierarchy.HierarchyTest.test_errno_mapping"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exception_hierarchy.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: HierarchyTest::test_errno_mapping (CPython 3.12 oracle)."""

import builtins
import errno
from errno import EEXIST


PEP_MAP = """
    +-- BlockingIOError        EAGAIN, EALREADY, EWOULDBLOCK, EINPROGRESS
    +-- ChildProcessError                                          ECHILD
    +-- ConnectionError
        +-- BrokenPipeError                              EPIPE, ESHUTDOWN
        +-- ConnectionAbortedError                           ECONNABORTED
        +-- ConnectionRefusedError                           ECONNREFUSED
        +-- ConnectionResetError                               ECONNRESET
    +-- FileExistsError                                            EEXIST
    +-- FileNotFoundError                                          ENOENT
    +-- InterruptedError                                            EINTR
    +-- IsADirectoryError                                          EISDIR
    +-- NotADirectoryError                                        ENOTDIR
    +-- PermissionError                        EACCES, EPERM, ENOTCAPABLE
    +-- ProcessLookupError                                          ESRCH
    +-- TimeoutError                                            ETIMEDOUT
"""


def make_map(source):
    mapping = {}
    for line in source.splitlines():
        line = line.strip("+- ")
        if not line:
            continue
        excname, _, errnames = line.partition(" ")
        for errname in filter(None, errnames.strip().split(", ")):
            if errname == "ENOTCAPABLE" and not hasattr(errno, errname):
                continue
            mapping[getattr(errno, errname)] = getattr(builtins, excname)
    return mapping


mapping = make_map(PEP_MAP)

e = OSError(EEXIST, "Bad file descriptor")
assert type(e) is FileExistsError

for errcode, exc in mapping.items():
    e = OSError(errcode, "Some message")
    assert type(e) is exc, (errcode, exc, type(e))

othercodes = set(errno.errorcode) - set(mapping)
for errcode in othercodes:
    e = OSError(errcode, "Some message")
    assert type(e) is OSError, repr(e)

print("HierarchyTest::test_errno_mapping: ok")
