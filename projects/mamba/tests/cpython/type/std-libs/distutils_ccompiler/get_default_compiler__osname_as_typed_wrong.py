# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_ccompiler"
# dimension = "type"
# case = "get_default_compiler__osname_as_typed_wrong"
# subject = "distutils.ccompiler.get_default_compiler(osname: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/ccompiler.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.ccompiler.get_default_compiler(osname: typed); call it with the wrong type.

typeshed contract: osname is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.ccompiler import get_default_compiler
try:
    get_default_compiler(_W())  # osname: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
