# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkgutil"
# dimension = "behavior"
# case = "pkgutil_tests__test_getdata_zipfile"
# subject = "cpython.test_pkgutil.PkgutilTests.test_getdata_zipfile"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pkgutil.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pkgutil.py::PkgutilTests::test_getdata_zipfile
"""Auto-ported test: PkgutilTests::test_getdata_zipfile (CPython 3.12 oracle)."""


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
z.writestr(pkg + '/sub/res.txt', RESOURCE_DATA)
z.close()
sys.path.insert(0, zip_file)
res1 = pkgutil.get_data(pkg, 'res.txt')

assert res1 == RESOURCE_DATA
res2 = pkgutil.get_data(pkg, 'sub/res.txt')

assert res2 == RESOURCE_DATA
names = []
for moduleinfo in pkgutil.iter_modules([zip_file]):

    assert isinstance(moduleinfo, pkgutil.ModuleInfo)
    names.append(moduleinfo.name)

assert names == ['test_getdata_zipfile']
del sys.path[0]
del sys.modules[pkg]
print("PkgutilTests::test_getdata_zipfile: ok")
