# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkgutil"
# dimension = "behavior"
# case = "pkgutil_tests__test_walkpackages_filesys"
# subject = "cpython.test_pkgutil.PkgutilTests.test_walkpackages_filesys"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pkgutil.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pkgutil.py::PkgutilTests::test_walkpackages_filesys
"""Auto-ported test: PkgutilTests::test_walkpackages_filesys (CPython 3.12 oracle)."""


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
pkg1 = 'test_walkpackages_filesys'
pkg1_dir = os.path.join(self_dirname, pkg1)
os.mkdir(pkg1_dir)
f = open(os.path.join(pkg1_dir, '__init__.py'), 'wb')
f.close()
os.mkdir(os.path.join(pkg1_dir, 'sub'))
f = open(os.path.join(pkg1_dir, 'sub', '__init__.py'), 'wb')
f.close()
f = open(os.path.join(pkg1_dir, 'sub', 'mod.py'), 'wb')
f.close()
pkg2 = 'sub'
pkg2_dir = os.path.join(self_dirname, pkg2)
os.mkdir(pkg2_dir)
f = open(os.path.join(pkg2_dir, '__init__.py'), 'wb')
f.close()
os.mkdir(os.path.join(pkg2_dir, 'test_walkpackages_filesys'))
f = open(os.path.join(pkg2_dir, 'test_walkpackages_filesys', '__init__.py'), 'wb')
f.close()
f = open(os.path.join(pkg2_dir, 'test_walkpackages_filesys', 'mod.py'), 'wb')
f.close()
expected = ['sub', 'sub.test_walkpackages_filesys', 'sub.test_walkpackages_filesys.mod', 'test_walkpackages_filesys', 'test_walkpackages_filesys.sub', 'test_walkpackages_filesys.sub.mod']
actual = [e[1] for e in pkgutil.walk_packages([self_dirname])]

assert actual == expected
for pkg in expected:
    if pkg.endswith('mod'):
        continue
    del sys.modules[pkg]
print("PkgutilTests::test_walkpackages_filesys: ok")
