# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "type"
# case = "getincrementaldecoder__encoding_as__BufferedEncoding_wrong"
# subject = "codecs.getincrementaldecoder(encoding: _BufferedEncoding)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/codecs.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: codecs.getincrementaldecoder(encoding: _BufferedEncoding); call it with the wrong type.

typeshed contract: encoding is _BufferedEncoding. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from codecs import getincrementaldecoder
try:
    getincrementaldecoder(_W())  # encoding: _BufferedEncoding <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
