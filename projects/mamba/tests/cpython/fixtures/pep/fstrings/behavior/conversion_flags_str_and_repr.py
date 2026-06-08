# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "conversion_flags_str_and_repr"
# subject = "fstring.conversion"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.conversion: !s uses str and !r uses repr: with v="it's alive", f'{v!s}' is v and f'{v!r}' is repr(v)"""
# !s / !r conversion flags select str / repr before formatting

_val = "it's alive"
assert f"{_val!s}" == _val, "!s = str(val)"
assert f"{_val!r}" == repr(_val), "!r = repr(val)"

print("conversion_flags_str_and_repr OK")
