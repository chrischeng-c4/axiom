# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compression_zstd"
# dimension = "type"
# case = "FrameInfo__init__decompressed_size_as_int_wrong"
# subject = "compression.zstd.FrameInfo.__init__(decompressed_size: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/compression/zstd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: compression.zstd.FrameInfo.__init__(decompressed_size: int); call it with the wrong type.

typeshed contract: decompressed_size is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from compression.zstd import FrameInfo
try:
    FrameInfo("not_an_int", 0)  # decompressed_size: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
