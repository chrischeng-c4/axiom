# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "type"
# case = "Match__groupdict__default_as__T_wrong"
# subject = "re.Match.groupdict(default: _T)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed default"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/re.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed default
# mamba-strict-type: TypeError
"""Type wall: re.Match.groupdict(default: _T); call it with the wrong type.

typeshed contract: default is _T. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from re import Match
obj = object.__new__(Match)
try:
    obj.groupdict(_W())  # default: _T <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
