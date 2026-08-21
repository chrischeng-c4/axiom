# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sysconfig"
# dimension = "behavior"
# case = "makefile_tests__test_parse_makefile"
# subject = "cpython.test_sysconfig.MakefileTests.test_parse_makefile"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sysconfig.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sysconfig.py::MakefileTests::test_parse_makefile
"""Auto-ported test: MakefileTests::test_parse_makefile (CPython 3.12 oracle)."""


import unittest
import sys
import os
import subprocess
import shutil
import json
import textwrap
from copy import copy
from test.support import captured_stdout, PythonSymlink, requires_subprocess, is_wasi
from test.support.import_helper import import_module
from test.support.os_helper import TESTFN, unlink, skip_unless_symlink, change_cwd
from test.support.venv import VirtualEnvironment
import sysconfig
from sysconfig import get_paths, get_platform, get_config_vars, get_path, get_path_names, _INSTALL_SCHEMES, get_default_scheme, get_scheme_names, get_config_var, _expand_vars, _get_preferred_schemes, _main
import _osx_support


HAS_USER_BASE = sysconfig._HAS_USER_BASE


# --- test body ---
pass
with open(TESTFN, 'w') as makefile:
    print('var1=a$(VAR2)', file=makefile)
    print('VAR2=b$(var3)', file=makefile)
    print('var3=42', file=makefile)
    print('var4=$/invalid', file=makefile)
    print('var5=dollar$$5', file=makefile)
    print('var6=${var3}/lib/python3.5/config-$(VAR2)$(var5)-x86_64-linux-gnu', file=makefile)
vars = sysconfig._parse_makefile(TESTFN)

assert vars == {'var1': 'ab42', 'VAR2': 'b42', 'var3': 42, 'var4': '$/invalid', 'var5': 'dollar$5', 'var6': '42/lib/python3.5/config-b42dollar$5-x86_64-linux-gnu'}
print("MakefileTests::test_parse_makefile: ok")
