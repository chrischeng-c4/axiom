# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fork1"
# dimension = "behavior"
# case = "fork_test__test_nested_import_lock_fork"
# subject = "cpython.test_fork1.ForkTest.test_nested_import_lock_fork"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fork1.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fork1.py::ForkTest::test_nested_import_lock_fork
"""Auto-ported test: ForkTest::test_nested_import_lock_fork."""


import _imp as imp
import os


if not hasattr(os, "fork"):
    print("ForkTest::test_nested_import_lock_fork: skipped fork unavailable")
else:
    exitcode = 42

    def fork_with_import_lock(level):
        release = 0
        in_child = False
        try:
            try:
                for _ in range(level):
                    imp.acquire_lock()
                    release += 1
                pid = os.fork()
                in_child = not pid
            finally:
                for _ in range(release):
                    imp.release_lock()
        except RuntimeError:
            if in_child:
                os._exit(1)
            raise
        if in_child:
            os._exit(exitcode)
        waited_pid, status = os.waitpid(pid, 0)
        assert waited_pid == pid
        assert os.WIFEXITED(status), status
        assert os.WEXITSTATUS(status) == exitcode, status

    for level in range(5):
        fork_with_import_lock(level)

    print("ForkTest::test_nested_import_lock_fork: ok")
