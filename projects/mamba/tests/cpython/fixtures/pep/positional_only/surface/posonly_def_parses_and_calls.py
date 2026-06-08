# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "positional_only"
# dimension = "surface"
# case = "posonly_def_parses_and_calls"
# subject = "/"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""/: a def with a `/` separator (def f(a, b, /)) parses and is callable positionally, returning the expected value"""

# PEP 570: parameters before `/` are positional-only. The def parses and the
# function is callable positionally.
def _add(a: int, b: int, /) -> int:
    return a + b

assert _add(1, 2) == 3, _add(1, 2)

print("posonly_def_parses_and_calls OK")
