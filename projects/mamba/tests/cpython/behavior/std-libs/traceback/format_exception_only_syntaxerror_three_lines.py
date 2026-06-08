# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "format_exception_only_syntaxerror_three_lines"
# subject = "traceback.format_exception_only"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.format_exception_only: a SyntaxError with (file, line, col, text) renders three lines: File location, the source line ('bad syntax'), then 'SyntaxError: error\\n'"""
import traceback

_se = SyntaxError("error", ("x.py", 23, None, "bad syntax"))
_se_lines = traceback.format_exception_only(SyntaxError, _se)
assert len(_se_lines) == 3, f"syntaxerror lines = {len(_se_lines)!r}"
assert _se_lines[1].strip() == "bad syntax", f"source line: {_se_lines[1]!r}"
assert _se_lines[-1] == "SyntaxError: error\n", f"final line: {_se_lines[-1]!r}"

print("format_exception_only_syntaxerror_three_lines OK")
