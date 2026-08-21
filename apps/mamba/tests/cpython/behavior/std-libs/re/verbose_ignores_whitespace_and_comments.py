# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "verbose_ignores_whitespace_and_comments"
# subject = "re.VERBOSE"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.VERBOSE: re.VERBOSE ignores unescaped whitespace and # comments in the pattern; the compiled pattern still reports the VERBOSE flag in .flags"""
import re

vp = re.compile(
    r"""
    \d{4}   # year
    -
    \d{2}   # month
    """,
    re.VERBOSE,
)
assert vp.fullmatch("2024-03") is not None, "verbose matches compact text"
assert vp.fullmatch("2024 - 03") is None, "verbose ignores pattern spaces only"
assert vp.flags & re.VERBOSE, "compiled flags include VERBOSE"

print("verbose_ignores_whitespace_and_comments OK")
