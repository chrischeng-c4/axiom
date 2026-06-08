# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "print_exc_writes_to_stream"
# subject = "traceback.print_exc"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.print_exc: print_exc(file=StringIO()) inside an except block writes the exception type 'KeyError' and message 'key_msg' to the given stream"""
import io
import traceback

_buf = io.StringIO()
try:
    raise KeyError("key_msg")
except KeyError:
    traceback.print_exc(file=_buf)
_out = _buf.getvalue()
assert "KeyError" in _out, f"print_exc KeyError: {_out!r}"
assert "key_msg" in _out, f"print_exc message: {_out!r}"

print("print_exc_writes_to_stream OK")
