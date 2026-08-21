# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "test_m_s_windows_t_c_p_flags__test_new_tcp_flags"
# subject = "cpython.test_socket.TestMSWindowsTCPFlags.test_new_tcp_flags"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("TestMSWindowsTCPFlags.test_new_tcp_flags", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestMSWindowsTCPFlags.test_new_tcp_flags did not pass"
print("TestMSWindowsTCPFlags::test_new_tcp_flags: ok")
