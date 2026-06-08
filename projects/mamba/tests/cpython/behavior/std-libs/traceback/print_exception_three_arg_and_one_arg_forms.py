# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "print_exception_three_arg_and_one_arg_forms"
# subject = "traceback.print_exception"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.print_exception: print_exception accepts both the 3-arg (type, value, None) and 1-arg (instance) forms; both emit 'Exception: projector\\n' to the stream"""
import io
import traceback

_o3 = io.StringIO()
traceback.print_exception(Exception, Exception("projector"), None, file=_o3)
assert _o3.getvalue() == "Exception: projector\n", f"3-arg print: {_o3.getvalue()!r}"
_o1 = io.StringIO()
traceback.print_exception(Exception("projector"), file=_o1)
assert _o1.getvalue() == "Exception: projector\n", f"1-arg print: {_o1.getvalue()!r}"

print("print_exception_three_arg_and_one_arg_forms OK")
