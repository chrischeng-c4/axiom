# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkgutil"
# dimension = "behavior"
# case = "pkgutil_tests__test_getdata_filesys"
# subject = "cpython.test_pkgutil.PkgutilTests.test_getdata_filesys"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pkgutil.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pkgutil.py::PkgutilTests::test_getdata_filesys
"""Auto-ported test: PkgutilTests::test_getdata_filesys (CPython 3.12 oracle)."""


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
pkg = 'test_getdata_filesys'
RESOURCE_DATA = b'Hello, world!\nSecond line\r\nThird line'
package_dir = os.path.join(self_dirname, pkg)
os.mkdir(package_dir)
f = open(os.path.join(package_dir, '__init__.py'), 'wb')
f.close()
f = open(os.path.join(package_dir, 'res.txt'), 'wb')
f.write(RESOURCE_DATA)
f.close()
os.mkdir(os.path.join(package_dir, 'sub'))
f = open(os.path.join(package_dir, 'sub', 'res.txt'), 'wb')
f.write(RESOURCE_DATA)
f.close()
res1 = pkgutil.get_data(pkg, 'res.txt')

assert res1 == RESOURCE_DATA
res2 = pkgutil.get_data(pkg, 'sub/res.txt')

assert res2 == RESOURCE_DATA
del sys.modules[pkg]
print("PkgutilTests::test_getdata_filesys: ok")
