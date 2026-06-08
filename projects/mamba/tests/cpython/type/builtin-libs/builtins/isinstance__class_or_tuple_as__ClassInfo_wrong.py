# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "isinstance__class_or_tuple_as__ClassInfo_wrong"
# subject = "builtins.isinstance(class_or_tuple: _ClassInfo)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed class_or_tuple"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed class_or_tuple
# mamba-strict-type: TypeError
"""Type wall: builtins.isinstance(class_or_tuple: _ClassInfo); call it with the wrong type.

typeshed contract: class_or_tuple is _ClassInfo. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


try:
    isinstance(None, _W())  # class_or_tuple: _ClassInfo <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
