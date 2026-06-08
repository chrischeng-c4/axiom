# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "u_random_tests__test_urandom_length"
# subject = "cpython.test_os.URandomTests.test_urandom_length"
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

def get_urandom_subprocess(count):
    code = '\n'.join(('import os, sys', 'data = os.urandom(%s)' % count, 'sys.stdout.buffer.write(data)', 'sys.stdout.buffer.flush()'))
    out = assert_python_ok('-c', code)
    stdout = out[1]
    assert len(stdout) == count
    return stdout
assert len(os.urandom(0)) == 0
assert len(os.urandom(1)) == 1
assert len(os.urandom(10)) == 10
assert len(os.urandom(100)) == 100
assert len(os.urandom(1000)) == 1000

print("URandomTests::test_urandom_length: ok")
