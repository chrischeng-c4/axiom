# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "type"
# case = "Writer__write__data_as__T_contra_wrong"
# subject = "io.Writer.write(data: _T_contra)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/io.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: io.Writer.write(data: _T_contra); call it with the wrong type.

typeshed contract: data is _T_contra. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from io import Writer
obj = object.__new__(Writer)
try:
    obj.write(_W())  # data: _T_contra <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
