# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect_traceback"
# dimension = "behavior"
# case = "print_exc_no_active_exception_no_raise"
# subject = "traceback.print_exc"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_traceback.py"
# status = "filled"
# ///
"""traceback.print_exc: traceback.print_exc() with no exception active does not raise; redirected to a StringIO it emits the 'NoneType: None' sentinel and returns None"""
import io
import traceback

buf = io.StringIO()
result = traceback.print_exc(file=buf)
assert result is None
assert "NoneType: None" in buf.getvalue(), repr(buf.getvalue())

print("print_exc_no_active_exception_no_raise OK")
