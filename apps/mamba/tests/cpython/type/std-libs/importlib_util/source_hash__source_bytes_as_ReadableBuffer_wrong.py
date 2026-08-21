# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_util"
# dimension = "type"
# case = "source_hash__source_bytes_as_ReadableBuffer_wrong"
# subject = "importlib.util.source_hash(source_bytes: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/util.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.util.source_hash(source_bytes: ReadableBuffer); call it with the wrong type.

typeshed contract: source_bytes is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.util import source_hash
try:
    source_hash(_W())  # source_bytes: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
