# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_ccompiler"
# dimension = "type"
# case = "CCompiler__compile__sources_as_Sequence_wrong"
# subject = "distutils.ccompiler.CCompiler.compile(sources: Sequence)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed sources"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/ccompiler.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed sources
# mamba-strict-type: TypeError
"""Type wall: distutils.ccompiler.CCompiler.compile(sources: Sequence); call it with the wrong type.

typeshed contract: sources is Sequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.ccompiler import CCompiler
obj = object.__new__(CCompiler)
try:
    obj.compile(_W())  # sources: Sequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
