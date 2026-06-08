# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ensurepip"
# dimension = "behavior"
# case = "test_packages__test_get_packages_with_dir"
# subject = "cpython.test_ensurepip.TestPackages.test_get_packages_with_dir"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ensurepip.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ensurepip.py::TestPackages::test_get_packages_with_dir
"""Auto-ported test: TestPackages::test_get_packages_with_dir (CPython 3.12 oracle)."""


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
def touch(directory, filename):
    fullname = os.path.join(directory, filename)
    open(fullname, 'wb').close()
pip_filename = 'pip-20.2.2-py2.py3-none-any.whl'
with tempfile.TemporaryDirectory() as tmpdir:
    touch(tmpdir, pip_filename)
    touch(tmpdir, 'wheel-0.34.2-py2.py3-none-any.whl')
    with unittest.mock.patch.object(ensurepip, '_PACKAGES', None), unittest.mock.patch.object(ensurepip, '_WHEEL_PKG_DIR', tmpdir):
        packages = ensurepip._get_packages()

    assert packages['pip'].version == '20.2.2'

    assert packages['pip'].wheel_path == os.path.join(tmpdir, pip_filename)

    assert sorted(packages) == ['pip']
print("TestPackages::test_get_packages_with_dir: ok")
