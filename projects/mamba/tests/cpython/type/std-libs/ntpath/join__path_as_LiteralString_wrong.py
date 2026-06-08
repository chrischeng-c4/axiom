# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ntpath"
# dimension = "type"
# case = "join__path_as_LiteralString_wrong"
# subject = "ntpath.join(path: LiteralString)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed path"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ntpath.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed path
# mamba-strict-type: TypeError
"""Type wall: ntpath.join(path: LiteralString); call it with the wrong type.

typeshed contract: path is LiteralString. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ntpath import join
try:
    join(_W())  # path: LiteralString <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
