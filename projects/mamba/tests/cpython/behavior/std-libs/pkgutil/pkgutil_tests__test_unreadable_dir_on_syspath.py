# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkgutil"
# dimension = "behavior"
# case = "pkgutil_tests__test_unreadable_dir_on_syspath"
# subject = "cpython.test_pkgutil.PkgutilTests.test_unreadable_dir_on_syspath"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pkgutil.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pkgutil.py::PkgutilTests::test_unreadable_dir_on_syspath
"""Auto-ported test: PkgutilTests::test_unreadable_dir_on_syspath (CPython 3.12 oracle)."""


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
package_name = 'unreadable_package'
d = os.path.join(self_dirname, package_name)
os.mkdir(d, 0)
pass
for t in pkgutil.walk_packages(path=[self_dirname]):

    raise AssertionError('unexpected package found')
print("PkgutilTests::test_unreadable_dir_on_syspath: ok")
