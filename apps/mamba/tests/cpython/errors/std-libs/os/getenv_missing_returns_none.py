# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "errors"
# case = "getenv_missing_returns_none"
# subject = "os.getenv"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
"""os.getenv: os.getenv on an undefined variable with no default returns None (no raise)"""
import os

result = os.getenv("NO_SUCH_VAR_XYZZY")
assert result is None, f"getenv on missing var should be None, got {result!r}"
print("getenv_missing_returns_none OK")
