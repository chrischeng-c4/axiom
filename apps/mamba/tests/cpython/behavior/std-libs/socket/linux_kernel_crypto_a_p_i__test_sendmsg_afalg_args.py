# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "linux_kernel_crypto_a_p_i__test_sendmsg_afalg_args"
# subject = "cpython.test_socket.LinuxKernelCryptoAPI.test_sendmsg_afalg_args"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("LinuxKernelCryptoAPI.test_sendmsg_afalg_args", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LinuxKernelCryptoAPI.test_sendmsg_afalg_args did not pass"
print("LinuxKernelCryptoAPI::test_sendmsg_afalg_args: ok")
