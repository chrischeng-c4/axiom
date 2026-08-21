# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "times_tests__test_times"
# subject = "cpython.test_os.TimesTests.test_times"
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
times = os.times()
assert isinstance(times, os.times_result)
for field in ('user', 'system', 'children_user', 'children_system', 'elapsed'):
    value = getattr(times, field)
    assert isinstance(value, float)
if os.name == 'nt':
    assert times.children_user == 0
    assert times.children_system == 0
    assert times.elapsed == 0

print("TimesTests::test_times: ok")
