# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ensurepip"
# dimension = "behavior"
# case = "test_bootstrap__test_basic_bootstrapping"
# subject = "cpython.test_ensurepip.TestBootstrap.test_basic_bootstrapping"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ensurepip.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ensurepip.py::TestBootstrap::test_basic_bootstrapping
"""Auto-ported test: TestBootstrap::test_basic_bootstrapping (CPython 3.12 oracle)."""


import contextlib
import os
import os.path
import sys
import tempfile
import test.support
import unittest
import unittest.mock
import ensurepip
import ensurepip._uninstall


class EnsurepipMixin:

    def setUp(self):
        run_pip_patch = unittest.mock.patch('ensurepip._run_pip')
        self.run_pip = run_pip_patch.start()
        self.run_pip.return_value = 0
        self.addCleanup(run_pip_patch.stop)
        real_devnull = os.devnull
        os_patch = unittest.mock.patch('ensurepip.os')
        patched_os = os_patch.start()
        patched_os.listdir = os.listdir
        self.addCleanup(os_patch.stop)
        patched_os.devnull = real_devnull
        patched_os.path = os.path
        self.os_environ = patched_os.environ = os.environ.copy()

@contextlib.contextmanager
def fake_pip(version=ensurepip.version()):
    if version is None:
        pip = None
    else:

        class FakePip:
            __version__ = version
        pip = FakePip()
    sentinel = object()
    orig_pip = sys.modules.get('pip', sentinel)
    sys.modules['pip'] = pip
    try:
        yield pip
    finally:
        if orig_pip is sentinel:
            del sys.modules['pip']
        else:
            sys.modules['pip'] = orig_pip

EXPECTED_VERSION_OUTPUT = 'pip ' + ensurepip.version()


# --- test body ---
run_pip_patch = unittest.mock.patch('ensurepip._run_pip')
self_run_pip = run_pip_patch.start()
self_run_pip.return_value = 0
pass
real_devnull = os.devnull
os_patch = unittest.mock.patch('ensurepip.os')
patched_os = os_patch.start()
patched_os.listdir = os.listdir
pass
patched_os.devnull = real_devnull
patched_os.path = os.path
self_os_environ = patched_os.environ = os.environ.copy()
ensurepip.bootstrap()
self_run_pip.assert_called_once_with(['install', '--no-cache-dir', '--no-index', '--find-links', unittest.mock.ANY, 'pip'], unittest.mock.ANY)
additional_paths = self_run_pip.call_args[0][1]

assert len(additional_paths) == 1
print("TestBootstrap::test_basic_bootstrapping: ok")
