# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "epoll"
# dimension = "behavior"
# case = "test_epoll__test_create"
# subject = "cpython.test_epoll.TestEPoll.test_create"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_epoll.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_epoll.py::TestEPoll::test_create
"""Auto-ported test: TestEPoll::test_create (CPython 3.12 oracle)."""


import errno
import select


if not hasattr(select, "epoll"):
    print("TestEPoll::test_create: skipped, select.epoll unavailable")
    raise SystemExit(0)

try:
    ep = select.epoll(16)
except OSError as exc:
    if exc.errno == errno.ENOSYS:
        print("TestEPoll::test_create: skipped, kernel epoll unavailable")
        raise SystemExit(0)
    raise AssertionError(str(exc)) from exc

assert ep.fileno() > 0, ep.fileno()
assert not ep.closed
ep.close()
assert ep.closed

try:
    ep.fileno()
except ValueError:
    pass
else:
    raise AssertionError("epoll.fileno() on closed epoll did not raise ValueError")

if hasattr(select, "EPOLL_CLOEXEC"):
    select.epoll(-1, select.EPOLL_CLOEXEC).close()
    select.epoll(flags=select.EPOLL_CLOEXEC).close()
    select.epoll(flags=0).close()

print("TestEPoll::test_create: ok")
