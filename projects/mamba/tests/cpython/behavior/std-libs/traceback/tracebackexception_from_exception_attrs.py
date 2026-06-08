# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "tracebackexception_from_exception_attrs"
# subject = "traceback.TracebackException"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.TracebackException: from_exception(e) of a ZeroDivisionError exposes exc_type, a matching str(), and a clean chain (__cause__/__context__ None, __suppress_context__ False)"""
import traceback

try:
    1 / 0
except ZeroDivisionError as e:
    exc_obj = e
    te = traceback.TracebackException.from_exception(e)
assert te.exc_type is ZeroDivisionError, f"exc_type = {te.exc_type!r}"
assert str(te) == str(exc_obj), f"str = {str(te)!r}"
assert te.__cause__ is None, f"cause = {te.__cause__!r}"
assert te.__context__ is None, f"context = {te.__context__!r}"
assert te.__suppress_context__ is False, f"suppress = {te.__suppress_context__!r}"

print("tracebackexception_from_exception_attrs OK")
