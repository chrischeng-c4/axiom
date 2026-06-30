# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "type"
# case = "disassemble__co_as__HaveCodeType_wrong"
# subject = "dis.disassemble(co: _HaveCodeType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/dis.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: dis.disassemble(co: _HaveCodeType); call it with the wrong type.

typeshed contract: co is _HaveCodeType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from dis import disassemble
try:
    disassemble(_W())  # co: _HaveCodeType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
