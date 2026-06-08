# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "stringio_iteration_yields_lines"
# subject = "io.StringIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.StringIO: iterating a StringIO yields each line (newline-terminated) in order"""
import io

_lines_buf = io.StringIO("line1\nline2\nline3\n")
_lines = list(_lines_buf)
assert _lines == ["line1\n", "line2\n", "line3\n"], f"iteration lines = {_lines!r}"

print("stringio_iteration_yields_lines OK")
