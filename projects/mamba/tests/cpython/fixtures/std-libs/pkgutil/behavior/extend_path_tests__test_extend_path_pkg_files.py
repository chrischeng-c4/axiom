# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkgutil"
# dimension = "behavior"
# case = "extend_path_tests__test_extend_path_pkg_files"
# subject = "cpython.test_pkgutil.ExtendPathTests.test_extend_path_pkg_files"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pkgutil.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pkgutil.py::ExtendPathTests::test_extend_path_pkg_files
"""Auto-ported test: ExtendPathTests::test_extend_path_pkg_files (CPython 3.12 oracle)."""


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
def create_init(pkgname):
    dirname = tempfile.mkdtemp()
    sys.path.insert(0, dirname)
    pkgdir = os.path.join(dirname, pkgname)
    os.mkdir(pkgdir)
    with open(os.path.join(pkgdir, '__init__.py'), 'w') as fl:
        fl.write('from pkgutil import extend_path\n__path__ = extend_path(__path__, __name__)\n')
    return dirname

def create_submodule(dirname, pkgname, submodule_name, value):
    module_name = os.path.join(dirname, pkgname, submodule_name + '.py')
    with open(module_name, 'w') as fl:
        print('value={}'.format(value), file=fl)
pkgname = 'foo'
dirname_0 = create_init(pkgname)
with open(os.path.join(dirname_0, 'bar.pkg'), 'w') as pkg_file:
    pkg_file.write('\n'.join(['baz', '/foo/bar/baz', '', '#comment']))
extended_paths = pkgutil.extend_path(sys.path, 'bar')

assert extended_paths[:-2] == sys.path

assert extended_paths[-2] == 'baz'

assert extended_paths[-1] == '/foo/bar/baz'
shutil.rmtree(dirname_0)
del sys.path[0]
print("ExtendPathTests::test_extend_path_pkg_files: ok")
