# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_io"
# dimension = "type"
# case = "BufferedWriter__init__raw_as__BufferedWriterStreamT_wrong"
# subject = "_io.BufferedWriter.__init__(raw: _BufferedWriterStreamT)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed raw"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_io.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed raw
# mamba-strict-type: TypeError
"""Type wall: _io.BufferedWriter.__init__(raw: _BufferedWriterStreamT); call it with the wrong type.

typeshed contract: raw is _BufferedWriterStreamT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _io import BufferedWriter
try:
    BufferedWriter(_W())  # raw: _BufferedWriterStreamT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
