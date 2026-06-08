# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_osx_support"
# dimension = "type"
# case = "compiler_fixup__compiler_so_as_Iterable_wrong"
# subject = "_osx_support.compiler_fixup(compiler_so: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_osx_support.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _osx_support.compiler_fixup(compiler_so: Iterable); call it with the wrong type.

typeshed contract: compiler_so is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _osx_support import compiler_fixup
try:
    compiler_fixup(_W(), None)  # compiler_so: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
