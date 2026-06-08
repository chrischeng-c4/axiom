# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "print_exc_no_active_exception_returns_none"
# subject = "traceback.print_exc"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.print_exc: print_exc(file=StringIO()) with no active exception returns None (the return-value contract is the parity oracle)"""
import io
import traceback

assert traceback.print_exc(file=io.StringIO()) is None

print("print_exc_no_active_exception_returns_none OK")
