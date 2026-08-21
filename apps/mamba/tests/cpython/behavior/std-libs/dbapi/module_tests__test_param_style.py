# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbapi"
# dimension = "behavior"
# case = "module_tests__test_param_style"
# subject = "cpython.test_dbapi.ModuleTests.test_param_style"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_dbapi.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import contextlib
import os
import sqlite3 as sqlite
import subprocess
import sys
import threading
import urllib.parse
import warnings
from _testcapi import INT_MAX, ULLONG_MAX
from os import SEEK_SET, SEEK_CUR, SEEK_END
assert sqlite.paramstyle == 'qmark'

print("ModuleTests::test_param_style: ok")
