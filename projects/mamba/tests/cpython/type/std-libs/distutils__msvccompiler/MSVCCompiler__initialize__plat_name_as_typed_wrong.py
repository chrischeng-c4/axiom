# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils__msvccompiler"
# dimension = "type"
# case = "MSVCCompiler__initialize__plat_name_as_typed_wrong"
# subject = "distutils._msvccompiler.MSVCCompiler.initialize(plat_name: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/_msvccompiler.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils._msvccompiler.MSVCCompiler.initialize(plat_name: typed); call it with the wrong type.

typeshed contract: plat_name is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils._msvccompiler import MSVCCompiler
obj = object.__new__(MSVCCompiler)
try:
    obj.initialize(_W())  # plat_name: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
