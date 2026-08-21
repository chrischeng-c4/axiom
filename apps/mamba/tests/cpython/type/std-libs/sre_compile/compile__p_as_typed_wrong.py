# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sre_compile"
# dimension = "type"
# case = "compile__p_as_typed_wrong"
# subject = "sre_compile.compile(p: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sre_compile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sre_compile.compile(p: typed); call it with the wrong type.

typeshed contract: p is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from sre_compile import compile
try:
    compile(_W())  # p: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
