# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "stringio_close_marks_closed"
# subject = "io.StringIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.StringIO: close() sets .closed True and a subsequent read() raises ValueError"""
import io

_c = io.StringIO("x")
_c.close()
assert _c.closed, "closed after close()"
_raised = False
try:
    _c.read()
except ValueError:
    _raised = True
assert _raised, "read on closed raises ValueError"

print("stringio_close_marks_closed OK")
