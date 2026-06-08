# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkgutil"
# dimension = "behavior"
# case = "pkgutil_tests__test_walk_packages_raises_on_string_or_bytes_input"
# subject = "cpython.test_pkgutil.PkgutilTests.test_walk_packages_raises_on_string_or_bytes_input"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pkgutil.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pkgutil.py::PkgutilTests::test_walk_packages_raises_on_string_or_bytes_input
"""Auto-ported test: PkgutilTests::test_walk_packages_raises_on_string_or_bytes_input (CPython 3.12 oracle)."""


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
str_input = 'test_dir'
try:
    list(pkgutil.walk_packages(str_input))
    raise AssertionError('expected (TypeError, ValueError)')
except (TypeError, ValueError):
    pass
bytes_input = b'test_dir'
try:
    list(pkgutil.walk_packages(bytes_input))
    raise AssertionError('expected (TypeError, ValueError)')
except (TypeError, ValueError):
    pass
print("PkgutilTests::test_walk_packages_raises_on_string_or_bytes_input: ok")
