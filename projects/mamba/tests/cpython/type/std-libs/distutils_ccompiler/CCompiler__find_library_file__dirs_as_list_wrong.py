# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_ccompiler"
# dimension = "type"
# case = "CCompiler__find_library_file__dirs_as_list_wrong"
# subject = "distutils.ccompiler.CCompiler.find_library_file(dirs: list)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed dirs"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/ccompiler.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed dirs
# mamba-strict-type: TypeError
"""Type wall: distutils.ccompiler.CCompiler.find_library_file(dirs: list); call it with the wrong type.

typeshed contract: dirs is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.ccompiler import CCompiler
obj = object.__new__(CCompiler)
try:
    obj.find_library_file(12345, "")  # dirs: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
