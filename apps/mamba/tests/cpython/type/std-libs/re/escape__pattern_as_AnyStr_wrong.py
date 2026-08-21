# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "type"
# case = "escape__pattern_as_AnyStr_wrong"
# subject = "re.escape(pattern: AnyStr)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed pattern"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/re.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed pattern
# mamba-strict-type: TypeError
"""Type wall: re.escape(pattern: AnyStr); call it with the wrong type.

typeshed contract: pattern is AnyStr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from re import escape
try:
    escape(_W())  # pattern: AnyStr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
