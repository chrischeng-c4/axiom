# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_compression"
# dimension = "type"
# case = "DecompressReader__init__fp_as__Reader_wrong"
# subject = "_compression.DecompressReader.__init__(fp: _Reader)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_compression.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _compression.DecompressReader.__init__(fp: _Reader); call it with the wrong type.

typeshed contract: fp is _Reader. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _compression import DecompressReader
try:
    DecompressReader(_W(), None)  # fp: _Reader <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
