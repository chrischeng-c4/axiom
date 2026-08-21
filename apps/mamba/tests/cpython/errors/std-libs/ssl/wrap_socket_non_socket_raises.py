# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "wrap_socket_non_socket_raises"
# subject = "ssl.SSLContext.wrap_socket"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.wrap_socket: wrap_socket_non_socket_raises (errors)."""
import ssl

_raised = False
try:
    ssl.create_default_context().wrap_socket(42)
except AttributeError:
    _raised = True
assert _raised, "wrap_socket_non_socket_raises: expected AttributeError"
print("wrap_socket_non_socket_raises OK")
