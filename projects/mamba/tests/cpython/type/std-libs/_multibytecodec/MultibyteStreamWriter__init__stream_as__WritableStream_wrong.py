# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_multibytecodec"
# dimension = "type"
# case = "MultibyteStreamWriter__init__stream_as__WritableStream_wrong"
# subject = "_multibytecodec.MultibyteStreamWriter.__init__(stream: _WritableStream)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_multibytecodec.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _multibytecodec.MultibyteStreamWriter.__init__(stream: _WritableStream); call it with the wrong type.

typeshed contract: stream is _WritableStream. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _multibytecodec import MultibyteStreamWriter
try:
    MultibyteStreamWriter(_W())  # stream: _WritableStream <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
