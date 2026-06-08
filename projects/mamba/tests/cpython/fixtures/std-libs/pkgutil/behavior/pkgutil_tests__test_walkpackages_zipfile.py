# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkgutil"
# dimension = "behavior"
# case = "pkgutil_tests__test_walkpackages_zipfile"
# subject = "cpython.test_pkgutil.PkgutilTests.test_walkpackages_zipfile"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pkgutil.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pkgutil.py::PkgutilTests::test_walkpackages_zipfile
"""Auto-ported test: PkgutilTests::test_walkpackages_zipfile (CPython 3.12 oracle)."""


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
'Tests the same as test_walkpackages_filesys, only with a zip file.'
zip = 'test_walkpackages_zipfile.zip'
pkg1 = 'test_walkpackages_zipfile'
pkg2 = 'sub'
zip_file = os.path.join(self_dirname, zip)
z = zipfile.ZipFile(zip_file, 'w')
z.writestr(pkg2 + '/__init__.py', '')
z.writestr(pkg2 + '/' + pkg1 + '/__init__.py', '')
z.writestr(pkg2 + '/' + pkg1 + '/mod.py', '')
z.writestr(pkg1 + '/__init__.py', '')
z.writestr(pkg1 + '/' + pkg2 + '/__init__.py', '')
z.writestr(pkg1 + '/' + pkg2 + '/mod.py', '')
z.close()
sys.path.insert(0, zip_file)
expected = ['sub', 'sub.test_walkpackages_zipfile', 'sub.test_walkpackages_zipfile.mod', 'test_walkpackages_zipfile', 'test_walkpackages_zipfile.sub', 'test_walkpackages_zipfile.sub.mod']
actual = [e[1] for e in pkgutil.walk_packages([zip_file])]

assert actual == expected
del sys.path[0]
for pkg in expected:
    if pkg.endswith('mod'):
        continue
    del sys.modules[pkg]
print("PkgutilTests::test_walkpackages_zipfile: ok")
