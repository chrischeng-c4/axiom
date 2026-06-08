# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_ccompiler"
# dimension = "type"
# case = "gen_lib_options__compiler_as_CCompiler_wrong"
# subject = "distutils.ccompiler.gen_lib_options(compiler: CCompiler)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/ccompiler.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.ccompiler.gen_lib_options(compiler: CCompiler); call it with the wrong type.

typeshed contract: compiler is CCompiler. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.ccompiler import gen_lib_options
try:
    gen_lib_options(_W(), None, None, None)  # compiler: CCompiler <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
