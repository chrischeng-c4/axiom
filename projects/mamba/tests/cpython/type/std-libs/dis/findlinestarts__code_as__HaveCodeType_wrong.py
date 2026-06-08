# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "type"
# case = "findlinestarts__code_as__HaveCodeType_wrong"
# subject = "dis.findlinestarts(code: _HaveCodeType)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed code"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/dis.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed code
# mamba-strict-type: TypeError
"""Type wall: dis.findlinestarts(code: _HaveCodeType); call it with the wrong type.

typeshed contract: code is _HaveCodeType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from dis import findlinestarts
try:
    findlinestarts(_W())  # code: _HaveCodeType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
