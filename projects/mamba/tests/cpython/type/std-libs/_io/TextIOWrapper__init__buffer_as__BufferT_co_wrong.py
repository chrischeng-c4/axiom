# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_io"
# dimension = "type"
# case = "TextIOWrapper__init__buffer_as__BufferT_co_wrong"
# subject = "_io.TextIOWrapper.__init__(buffer: _BufferT_co)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_io.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _io.TextIOWrapper.__init__(buffer: _BufferT_co); call it with the wrong type.

typeshed contract: buffer is _BufferT_co. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _io import TextIOWrapper
try:
    TextIOWrapper(_W())  # buffer: _BufferT_co <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
