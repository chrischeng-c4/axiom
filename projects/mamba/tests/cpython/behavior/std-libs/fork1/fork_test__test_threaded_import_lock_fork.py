# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fork1"
# dimension = "behavior"
# case = "fork_test__test_threaded_import_lock_fork"
# subject = "cpython.test_fork1.ForkTest.test_threaded_import_lock_fork"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fork1.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fork1.py::ForkTest::test_threaded_import_lock_fork
"""Auto-ported test: ForkTest::test_threaded_import_lock_fork."""


import _imp as imp
import os
import signal
import sys
import threading
import time


if not hasattr(os, "fork"):
    print("ForkTest::test_threaded_import_lock_fork: skipped fork unavailable")
else:
    import_started = threading.Event()
    fake_module_name = "fake test module"
    partial_module = "partial"
    complete_module = "complete"

    def importer():
        imp.acquire_lock()
        try:
            sys.modules[fake_module_name] = partial_module
            import_started.set()
            time.sleep(0.01)
            sys.modules[fake_module_name] = complete_module
        finally:
            imp.release_lock()

    t = threading.Thread(target=importer)
    t.start()
    import_started.wait()
    exitcode = 42
    pid = os.fork()
    try:
        if not pid:
            module = __import__(fake_module_name)
            os._exit(exitcode if module == complete_module else 1)
        t.join()
        waited_pid, status = os.waitpid(pid, 0)
        assert waited_pid == pid
        assert os.WIFEXITED(status), status
        assert os.WEXITSTATUS(status) == exitcode, status
    finally:
        try:
            os.kill(pid, signal.SIGKILL)
        except OSError:
            pass

    print("ForkTest::test_threaded_import_lock_fork: ok")
