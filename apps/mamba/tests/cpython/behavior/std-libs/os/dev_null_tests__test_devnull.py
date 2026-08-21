# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "dev_null_tests__test_devnull"
# subject = "cpython.test_os.DevNullTests.test_devnull"
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
with open(os.devnull, 'wb', 0) as f:
    f.write(b'hello')
    f.close()
with open(os.devnull, 'rb') as f:
    assert f.read() == b''

print("DevNullTests::test_devnull: ok")
