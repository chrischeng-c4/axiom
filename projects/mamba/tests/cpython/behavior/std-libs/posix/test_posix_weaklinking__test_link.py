# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posix"
# dimension = "behavior"
# case = "test_posix_weaklinking__test_link"
# subject = "cpython.test_posix.TestPosixWeaklinking.test_link"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_posix.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_posix.py::TestPosixWeaklinking::test_link
"""Auto-ported test: TestPosixWeaklinking::test_link (CPython 3.12 oracle)."""


from test import support
from test.support import import_helper
from test.support import os_helper
from test.support import warnings_helper
from test.support.script_helper import assert_python_ok
import copy
import errno
import sys
import signal
import time
import os
import platform
import pickle
import stat
import tempfile
import unittest
import warnings
import textwrap
from contextlib import contextmanager


'Test posix functions'

try:
    import posix
except ImportError:
    import nt as posix

try:
    import pwd
except ImportError:
    pwd = None

_DUMMY_SYMLINK = os.path.join(tempfile.gettempdir(), os_helper.TESTFN + '-dummy-symlink')

requires_32b = unittest.skipUnless(sys.maxsize < 2 ** 32 and (not (support.is_emscripten or support.is_wasi)), 'test is only meaningful on 32-bit builds')

def _supports_sched():
    if not hasattr(posix, 'sched_getscheduler'):
        return False
    try:
        posix.sched_getscheduler(0)
    except OSError as e:
        if e.errno == errno.ENOSYS:
            return False
    return True

requires_sched = unittest.skipUnless(_supports_sched(), 'requires POSIX scheduler API')

def tearDownModule():
    support.reap_children()


# --- test body ---
def _verify_available(name):
    if name not in self_available:
        raise unittest.SkipTest(f'{name} not weak-linked')
import sysconfig
import platform
config_vars = sysconfig.get_config_vars()
self_available = {nm for nm in config_vars if nm.startswith('HAVE_') and config_vars[nm]}
self_mac_ver = tuple((int(part) for part in platform.mac_ver()[0].split('.')))
_verify_available('HAVE_LINKAT')
if self_mac_ver >= (10, 10):

    assert 'HAVE_LINKAT' in posix._have_functions
else:

    assert 'HAVE_LINKAT' not in posix._have_functions
    try:
        os.link('source', 'target', src_dir_fd=0)
        raise AssertionError('expected NotImplementedError')
    except NotImplementedError as _aR_e:
        import re as _re_aR
        assert _re_aR.search('src_dir_fd unavailable', str(_aR_e))
    try:
        os.link('source', 'target', dst_dir_fd=0)
        raise AssertionError('expected NotImplementedError')
    except NotImplementedError as _aR_e:
        import re as _re_aR
        assert _re_aR.search('dst_dir_fd unavailable', str(_aR_e))
    try:
        os.link('source', 'target', src_dir_fd=0, dst_dir_fd=0)
        raise AssertionError('expected NotImplementedError')
    except NotImplementedError as _aR_e:
        import re as _re_aR
        assert _re_aR.search('src_dir_fd unavailable', str(_aR_e))
    with os_helper.temp_dir() as base_path:
        link_path = os.path.join(base_path, 'link')
        target_path = os.path.join(base_path, 'target')
        source_path = os.path.join(base_path, 'source')
        with open(source_path, 'w') as fp:
            fp.write('data')
        os.symlink('target', link_path)
        try:
            os.link(source_path, link_path, follow_symlinks=True)
            raise AssertionError('expected FileExistsError')
        except FileExistsError:
            pass
        try:
            os.link(source_path, link_path, follow_symlinks=False)
            raise AssertionError('expected FileExistsError')
        except FileExistsError:
            pass
print("TestPosixWeaklinking::test_link: ok")
