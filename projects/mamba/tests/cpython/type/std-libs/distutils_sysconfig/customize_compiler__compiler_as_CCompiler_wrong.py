# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_sysconfig"
# dimension = "type"
# case = "customize_compiler__compiler_as_CCompiler_wrong"
# subject = "distutils.sysconfig.customize_compiler(compiler: CCompiler)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/sysconfig.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.sysconfig.customize_compiler(compiler: CCompiler); call it with the wrong type.

typeshed contract: compiler is CCompiler. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.sysconfig import customize_compiler
try:
    customize_compiler(_W())  # compiler: CCompiler <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
