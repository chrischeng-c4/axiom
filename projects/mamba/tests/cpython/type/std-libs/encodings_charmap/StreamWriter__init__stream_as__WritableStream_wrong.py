# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_charmap"
# dimension = "type"
# case = "StreamWriter__init__stream_as__WritableStream_wrong"
# subject = "encodings.charmap.StreamWriter.__init__(stream: _WritableStream)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/charmap.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.charmap.StreamWriter.__init__(stream: _WritableStream); call it with the wrong type.

typeshed contract: stream is _WritableStream. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from encodings.charmap import StreamWriter
try:
    StreamWriter(_W())  # stream: _WritableStream <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
