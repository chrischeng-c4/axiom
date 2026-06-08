# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "misc_test__test_levenshtein_distance"
# subject = "cpython.test_traceback.MiscTest.test_levenshtein_distance"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_traceback.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from collections import namedtuple
from io import StringIO
import linecache
import sys
import types
import inspect
import builtins
import re
import tempfile
import random
import string
import shutil
import json
import textwrap
import traceback
from functools import partial
from pathlib import Path

def CHECK(a, b, expected):
    actual = traceback._levenshtein_distance(a, b, 4044)
    assert actual == expected
CHECK('', '', 0)
CHECK('', 'a', 2)
CHECK('a', 'A', 1)
CHECK('Apple', 'Aple', 2)
CHECK('Banana', 'B@n@n@', 6)
CHECK('Cherry', 'Cherry!', 2)
CHECK('---0---', '------', 2)
CHECK('abc', 'y', 6)
CHECK('aa', 'bb', 4)
CHECK('aaaaa', 'AAAAA', 5)
CHECK('wxyz', 'wXyZ', 2)
CHECK('wxyz', 'wXyZ123', 8)
CHECK('Python', 'Java', 12)
CHECK('Java', 'C#', 8)
CHECK('AbstractFoobarManager', 'abstract_foobar_manager', 3 + 2 * 2)
CHECK('CPython', 'PyPy', 10)
CHECK('CPython', 'pypy', 11)
CHECK('AttributeError', 'AttributeErrop', 2)
CHECK('AttributeError', 'AttributeErrorTests', 10)
CHECK('ABA', 'AAB', 4)

print("MiscTest::test_levenshtein_distance: ok")
