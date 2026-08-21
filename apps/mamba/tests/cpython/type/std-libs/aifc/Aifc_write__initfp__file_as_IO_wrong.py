# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "aifc"
# dimension = "type"
# case = "Aifc_write__initfp__file_as_IO_wrong"
# subject = "aifc.Aifc_write.initfp(file: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/aifc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: aifc.Aifc_write.initfp(file: IO); call it with the wrong type.

typeshed contract: file is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from aifc import Aifc_write
obj = object.__new__(Aifc_write)
try:
    obj.initfp(_W())  # file: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
