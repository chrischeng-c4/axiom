# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "tracebackexception_equality_between_equivalent_captures"
# subject = "traceback.TracebackException"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.TracebackException: two from_exception captures of the same exception are distinct objects (is not) yet compare equal (==), and differ from an unrelated object"""
import traceback

try:
    1 / 0
except ZeroDivisionError as e:
    te = traceback.TracebackException.from_exception(e)
    te_again = traceback.TracebackException.from_exception(e)
assert te is not te_again, "captures are distinct objects"
assert te == te_again, "equivalent captures compare equal"
assert te != object(), "unrelated object compares unequal"

print("tracebackexception_equality_between_equivalent_captures OK")
