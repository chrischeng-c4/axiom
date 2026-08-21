# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "type"
# case = "compile__pattern_as_Pattern_wrong"
# subject = "re.compile(pattern: Pattern)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/re.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: re.compile(pattern: Pattern); call it with the wrong type.

typeshed contract: pattern is Pattern. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from re import compile
try:
    compile(_W())  # pattern: Pattern <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
