# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "sslerror_str_and_errno"
# subject = "ssl.SSLError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLError: ssl.SSLError(1, 'foo') exposes its message via str() ('foo') and the errno argument (1); SSLZeroReturnError behaves the same"""
import ssl

_err = ssl.SSLError(1, "foo")
assert str(_err) == "foo", f"SSLError str = {_err}"
assert _err.errno == 1, f"SSLError errno = {_err.errno}"
_zero = ssl.SSLZeroReturnError(1, "foo")
assert str(_zero) == "foo" and _zero.errno == 1, "ZeroReturn str/errno"

print("sslerror_str_and_errno OK")
