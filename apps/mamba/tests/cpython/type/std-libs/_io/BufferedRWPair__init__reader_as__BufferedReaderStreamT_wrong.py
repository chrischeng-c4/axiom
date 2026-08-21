# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_io"
# dimension = "type"
# case = "BufferedRWPair__init__reader_as__BufferedReaderStreamT_wrong"
# subject = "_io.BufferedRWPair.__init__(reader: _BufferedReaderStreamT)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_io.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _io.BufferedRWPair.__init__(reader: _BufferedReaderStreamT); call it with the wrong type.

typeshed contract: reader is _BufferedReaderStreamT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _io import BufferedRWPair
try:
    BufferedRWPair(_W(), None)  # reader: _BufferedReaderStreamT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
