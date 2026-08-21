# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "type"
# case = "read_float8__f_as_IO_wrong"
# subject = "pickletools.read_float8(f: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickletools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickletools.read_float8(f: IO); call it with the wrong type.

typeshed contract: f is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pickletools import read_float8
try:
    read_float8(_W())  # f: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
