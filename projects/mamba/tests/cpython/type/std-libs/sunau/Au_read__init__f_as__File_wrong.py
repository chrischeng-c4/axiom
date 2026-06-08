# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sunau"
# dimension = "type"
# case = "Au_read__init__f_as__File_wrong"
# subject = "sunau.Au_read.__init__(f: _File)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sunau.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sunau.Au_read.__init__(f: _File); call it with the wrong type.

typeshed contract: f is _File. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from sunau import Au_read
try:
    Au_read(_W())  # f: _File <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
