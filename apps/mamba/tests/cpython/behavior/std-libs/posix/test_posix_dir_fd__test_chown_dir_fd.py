# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posix"
# dimension = "behavior"
# case = "test_posix_dir_fd__test_chown_dir_fd"
# subject = "cpython.test_posix.TestPosixDirFd.test_chown_dir_fd"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_posix.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_posix
_suite = unittest.defaultTestLoader.loadTestsFromName("TestPosixDirFd.test_chown_dir_fd", test_posix)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestPosixDirFd.test_chown_dir_fd did not pass"
print("TestPosixDirFd::test_chown_dir_fd: ok")
