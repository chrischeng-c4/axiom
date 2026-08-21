# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "p_o_s_i_x_process_test_case__test_close_fds_when_max_fd_is_lowered"
# subject = "cpython.test_subprocess.POSIXProcessTestCase.test_close_fds_when_max_fd_is_lowered"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_subprocess
_suite = unittest.defaultTestLoader.loadTestsFromName("POSIXProcessTestCase.test_close_fds_when_max_fd_is_lowered", test_subprocess)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython POSIXProcessTestCase.test_close_fds_when_max_fd_is_lowered did not pass"
print("POSIXProcessTestCase::test_close_fds_when_max_fd_is_lowered: ok")
