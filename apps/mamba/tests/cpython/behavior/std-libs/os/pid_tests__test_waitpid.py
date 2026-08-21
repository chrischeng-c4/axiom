# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "pid_tests__test_waitpid"
# subject = "cpython.test_os.PidTests.test_waitpid"
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

def check_waitpid(code, exitcode, callback=None):
    if sys.platform == 'win32':
        args = [f'"{sys.executable}"', '-c', f'"{code}"']
    else:
        args = [sys.executable, '-c', code]
    pid = os.spawnv(os.P_NOWAIT, sys.executable, args)
    if callback is not None:
        callback(pid)
    pid2, status = os.waitpid(pid, 0)
    assert os.waitstatus_to_exitcode(status) == exitcode
    assert pid2 == pid
check_waitpid(code='pass', exitcode=0)

print("PidTests::test_waitpid: ok")
