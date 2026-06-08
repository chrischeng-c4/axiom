# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "exec_tests__test_execv_with_bad_arglist"
# subject = "cpython.test_os.ExecTests.test_execv_with_bad_arglist"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import asyncio
import codecs
import contextlib
import decimal
import errno
import fnmatch
import fractions
import itertools
import locale
import os
import pickle
import select
import shutil
import signal
import socket
import stat
import struct
import subprocess
import sys
import sysconfig
import tempfile
import textwrap
import time
import types
import uuid
import warnings
from platform import win32_is_iot
try:
    os.execv('notepad', ())
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass
try:
    os.execv('notepad', [])
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass
try:
    os.execv('notepad', ('',))
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass
try:
    os.execv('notepad', [''])
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass

print("ExecTests::test_execv_with_bad_arglist: ok")
