# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "format_exception_only_bare_baseexception_type_only"
# subject = "traceback.format_exception_only"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.format_exception_only: a BaseException with no message (KeyboardInterrupt()) renders type-only ['KeyboardInterrupt\\n'] with no trailing ': message'"""
import traceback

_kbi = KeyboardInterrupt()
assert traceback.format_exception_only(_kbi.__class__, _kbi) == ["KeyboardInterrupt\n"], \
    "bare BaseException renders type-only"

print("format_exception_only_bare_baseexception_type_only OK")
