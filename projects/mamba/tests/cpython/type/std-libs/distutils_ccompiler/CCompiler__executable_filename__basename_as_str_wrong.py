# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_ccompiler"
# dimension = "type"
# case = "CCompiler__executable_filename__basename_as_str_wrong"
# subject = "distutils.ccompiler.CCompiler.executable_filename(basename: str)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed basename"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/ccompiler.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed basename
# mamba-strict-type: TypeError
"""Type wall: distutils.ccompiler.CCompiler.executable_filename(basename: str); call it with the wrong type.

typeshed contract: basename is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.ccompiler import CCompiler
obj = object.__new__(CCompiler)
try:
    obj.executable_filename(12345)  # basename: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
