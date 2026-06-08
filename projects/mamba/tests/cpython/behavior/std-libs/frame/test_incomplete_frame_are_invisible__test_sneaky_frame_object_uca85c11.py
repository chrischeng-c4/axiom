# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "frame"
# dimension = "behavior"
# case = "test_incomplete_frame_are_invisible__test_sneaky_frame_object_uca85c11"
# subject = "cpython.test_frame.TestIncompleteFrameAreInvisible.test_sneaky_frame_object"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_frame.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_frame
_suite = unittest.defaultTestLoader.loadTestsFromName("TestIncompleteFrameAreInvisible.test_sneaky_frame_object", test_frame)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestIncompleteFrameAreInvisible.test_sneaky_frame_object did not pass"
print("TestIncompleteFrameAreInvisible::test_sneaky_frame_object: ok")
