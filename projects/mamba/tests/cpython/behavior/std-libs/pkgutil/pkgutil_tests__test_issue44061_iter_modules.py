# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkgutil"
# dimension = "behavior"
# case = "pkgutil_tests__test_issue44061_iter_modules"
# subject = "cpython.test_pkgutil.PkgutilTests.test_issue44061_iter_modules"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pkgutil.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pkgutil.py::PkgutilTests::test_issue44061_iter_modules
"""Auto-ported test: PkgutilTests::test_issue44061_iter_modules (CPython 3.12 oracle)."""


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
zip = 'test_getdata_zipfile.zip'
pkg = 'test_getdata_zipfile'
RESOURCE_DATA = b'Hello, world!\nSecond line\r\nThird line'
zip_file = os.path.join(self_dirname, zip)
z = zipfile.ZipFile(zip_file, 'w')
z.writestr(pkg + '/__init__.py', '')
z.writestr(pkg + '/res.txt', RESOURCE_DATA)
z.close()
sys.path.insert(0, zip_file)
try:
    res = pkgutil.get_data(pkg, 'res.txt')

    assert res == RESOURCE_DATA
    names = []
    for moduleinfo in pkgutil.iter_modules([FakePath(zip_file)]):

        assert isinstance(moduleinfo, pkgutil.ModuleInfo)
        names.append(moduleinfo.name)

    assert names == [pkg]
finally:
    del sys.path[0]
    sys.modules.pop(pkg, None)
expected_msg = 'path must be None or list of paths to look for modules in'
try:
    list(pkgutil.iter_modules('invalid_path'))
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search(expected_msg, str(_aR_e))
print("PkgutilTests::test_issue44061_iter_modules: ok")
