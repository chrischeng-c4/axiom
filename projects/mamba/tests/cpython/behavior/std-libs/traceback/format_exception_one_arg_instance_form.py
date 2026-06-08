# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "format_exception_one_arg_instance_form"
# subject = "traceback.format_exception"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.format_exception: the 3.12 single-arg form accepts a bare exception instance: format_exception(Exception('projector')) and format_exception_only(...) both yield ['Exception: projector\\n']"""
import traceback

_ex = Exception("projector")
assert traceback.format_exception(_ex) == ["Exception: projector\n"], "1-arg format_exception"
assert traceback.format_exception_only(_ex) == ["Exception: projector\n"], "1-arg format_exception_only"

print("format_exception_one_arg_instance_form OK")
