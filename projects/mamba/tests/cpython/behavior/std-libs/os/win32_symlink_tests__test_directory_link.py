# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "win32_symlink_tests__test_directory_link"
# subject = "cpython.test_os.Win32SymlinkTests.test_directory_link"
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
filelink = 'filelinktest'
filelink_target = os.path.abspath(__file__)
dirlink = 'dirlinktest'
dirlink_target = os.path.dirname(filelink_target)
missing_link = 'missing link'

def _create_missing_dir_link():
    """Create a "directory" link to a non-existent target"""
    linkname = missing_link
    if os.path.lexists(linkname):
        os.remove(linkname)
    target = 'c:\\\\target does not exist.29r3c740'
    assert not os.path.exists(target)
    target_is_dir = True
    os.symlink(target, linkname, target_is_dir)

def check_stat(link, target):
    assert os.stat(link) == os.stat(target)
    assert os.lstat(link) != os.stat(link)
    bytes_link = os.fsencode(link)
    assert os.stat(bytes_link) == os.stat(target)
    assert os.lstat(bytes_link) != os.stat(bytes_link)
assert os.path.exists(dirlink_target)
assert os.path.exists(filelink_target)
assert not os.path.exists(dirlink)
assert not os.path.exists(filelink)
assert not os.path.exists(missing_link)
os.symlink(dirlink_target, dirlink)
assert os.path.exists(dirlink)
assert os.path.isdir(dirlink)
assert os.path.islink(dirlink)
check_stat(dirlink, dirlink_target)

print("Win32SymlinkTests::test_directory_link: ok")
