# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posix"
# dimension = "behavior"
# case = "test_posix_spawn_p__test_posix_spawnp"
# subject = "cpython.test_posix.TestPosixSpawnP.test_posix_spawnp"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_posix.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_posix
_suite = unittest.defaultTestLoader.loadTestsFromName("TestPosixSpawnP.test_posix_spawnp", test_posix)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestPosixSpawnP.test_posix_spawnp did not pass"
print("TestPosixSpawnP::test_posix_spawnp: ok")
