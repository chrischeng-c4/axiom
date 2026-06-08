# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_ccompiler"
# dimension = "type"
# case = "CCompiler__object_filenames__source_filenames_as_Iterable_wrong"
# subject = "distutils.ccompiler.CCompiler.object_filenames(source_filenames: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/ccompiler.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.ccompiler.CCompiler.object_filenames(source_filenames: Iterable); call it with the wrong type.

typeshed contract: source_filenames is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.ccompiler import CCompiler
obj = object.__new__(CCompiler)
try:
    obj.object_filenames(_W())  # source_filenames: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
