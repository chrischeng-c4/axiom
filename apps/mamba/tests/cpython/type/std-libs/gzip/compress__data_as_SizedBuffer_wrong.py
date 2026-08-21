# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "type"
# case = "compress__data_as_SizedBuffer_wrong"
# subject = "gzip.compress(data: SizedBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/gzip.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: gzip.compress(data: SizedBuffer); call it with the wrong type.

typeshed contract: data is SizedBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from gzip import compress
try:
    compress(_W())  # data: SizedBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
