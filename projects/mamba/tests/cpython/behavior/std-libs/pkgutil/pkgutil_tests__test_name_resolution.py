# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkgutil"
# dimension = "behavior"
# case = "pkgutil_tests__test_name_resolution"
# subject = "cpython.test_pkgutil.PkgutilTests.test_name_resolution"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pkgutil.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pkgutil.py::PkgutilTests::test_name_resolution
"""Auto-ported test: PkgutilTests::test_name_resolution (CPython 3.12 oracle)."""


from pathlib import Path
from test.support.import_helper import unload, CleanImport
from test.support.warnings_helper import check_warnings, ignore_warnings
import unittest
import sys
import importlib
from importlib.util import spec_from_file_location
import pkgutil
import os
import os.path
import tempfile
import shutil
import zipfile
from test.support.import_helper import DirsOnSysPath
from test.support.os_helper import FakePath
from test.test_importlib.util import uncache


def tearDownModule():
    import zipimport
    import importlib
    zipimport._zip_directory_cache.clear()
    importlib.invalidate_caches()


# --- test body ---
self_dirname = tempfile.mkdtemp()
pass
sys.path.insert(0, self_dirname)
import logging
import logging.handlers
success_cases = (('os', os), ('os.path', os.path), ('os.path:pathsep', os.path.pathsep), ('logging', logging), ('logging:', logging), ('logging.handlers', logging.handlers), ('logging.handlers:', logging.handlers), ('logging.handlers:SysLogHandler', logging.handlers.SysLogHandler), ('logging.handlers.SysLogHandler', logging.handlers.SysLogHandler), ('logging.handlers:SysLogHandler.LOG_ALERT', logging.handlers.SysLogHandler.LOG_ALERT), ('logging.handlers.SysLogHandler.LOG_ALERT', logging.handlers.SysLogHandler.LOG_ALERT), ('builtins.int', int), ('builtins:int', int), ('builtins.int.from_bytes', int.from_bytes), ('builtins:int.from_bytes', int.from_bytes), ('builtins.ZeroDivisionError', ZeroDivisionError), ('builtins:ZeroDivisionError', ZeroDivisionError), ('os:path', os.path))
failure_cases = ((None, TypeError), (1, TypeError), (2.0, TypeError), (True, TypeError), ('', ValueError), ('?abc', ValueError), ('abc/foo', ValueError), ('foo', ImportError), ('os.foo', AttributeError), ('os.foo:', ImportError), ('os.pth:pathsep', ImportError), ('logging.handlers:NoSuchHandler', AttributeError), ('logging.handlers:SysLogHandler.NO_SUCH_VALUE', AttributeError), ('logging.handlers.SysLogHandler.NO_SUCH_VALUE', AttributeError), ('ZeroDivisionError', ImportError), ('os.path.9abc', ValueError), ('9abc', ValueError))
unicode_words = ('वमस', 'é', 'È', '안녕하세요', 'さよなら', 'ありがとう', 'Хорошо', 'спасибо', '现代汉语常用字表')
for uw in unicode_words:
    d = os.path.join(self_dirname, uw)
    try:
        os.makedirs(d, exist_ok=True)
    except UnicodeEncodeError:
        continue
    f = os.path.join(d, '__init__.py')
    with open(f, 'w') as f:
        f.write('')
        f.flush()
    importlib.invalidate_caches()
    mod = importlib.import_module(uw)
    success_cases += ((uw, mod),)
    if len(uw) > 1:
        failure_cases += ((uw[:-1], ImportError),)
failure_cases += (('०वमस', ValueError),)
for s, expected in success_cases:
    o = pkgutil.resolve_name(s)

    assert o == expected
for s, exc in failure_cases:
    try:
        pkgutil.resolve_name(s)
        raise AssertionError('expected exc')
    except exc:
        pass
print("PkgutilTests::test_name_resolution: ok")
