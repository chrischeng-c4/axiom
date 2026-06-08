# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "type"
# case = "replace__obj_as__SupportsReplace_wrong"
# subject = "copy.replace(obj: _SupportsReplace)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/copy.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: copy.replace(obj: _SupportsReplace); call it with the wrong type.

typeshed contract: obj is _SupportsReplace. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from copy import replace
try:
    replace(_W())  # obj: _SupportsReplace <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
