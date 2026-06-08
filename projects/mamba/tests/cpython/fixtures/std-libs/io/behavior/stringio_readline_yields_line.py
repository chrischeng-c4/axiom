# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "stringio_readline_yields_line"
# subject = "io.StringIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.StringIO: readline() returns the next line including its trailing newline"""
import io

_lines = io.StringIO("line1\nline2\nline3")
assert _lines.readline() == "line1\n", "readline keeps trailing newline"

print("stringio_readline_yields_line OK")
