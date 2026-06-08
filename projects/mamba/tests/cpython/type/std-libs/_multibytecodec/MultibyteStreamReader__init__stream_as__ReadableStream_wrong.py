# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_multibytecodec"
# dimension = "type"
# case = "MultibyteStreamReader__init__stream_as__ReadableStream_wrong"
# subject = "_multibytecodec.MultibyteStreamReader.__init__(stream: _ReadableStream)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_multibytecodec.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _multibytecodec.MultibyteStreamReader.__init__(stream: _ReadableStream); call it with the wrong type.

typeshed contract: stream is _ReadableStream. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _multibytecodec import MultibyteStreamReader
try:
    MultibyteStreamReader(_W())  # stream: _ReadableStream <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
