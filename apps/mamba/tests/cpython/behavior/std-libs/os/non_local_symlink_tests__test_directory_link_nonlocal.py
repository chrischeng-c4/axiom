# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "non_local_symlink_tests__test_directory_link_nonlocal"
# subject = "cpython.test_os.NonLocalSymlinkTests.test_directory_link_nonlocal"
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
'\n        Create this structure:\n\n        base\n         \\___ some_dir\n        '
os.makedirs('base/some_dir')
'\n        The symlink target should resolve relative to the link, not relative\n        to the current directory.\n\n        Then, link base/some_link -> base/some_dir and ensure that some_link\n        is resolved as a directory.\n\n        In issue13772, it was discovered that directory detection failed if\n        the symlink target was not specified relative to the current\n        directory, which was a defect in the implementation.\n        '
src = os.path.join('base', 'some_link')
os.symlink('some_dir', src)
assert os.path.isdir(src)

print("NonLocalSymlinkTests::test_directory_link_nonlocal: ok")
