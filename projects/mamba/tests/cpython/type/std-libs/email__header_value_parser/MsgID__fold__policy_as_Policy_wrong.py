# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "MsgID__fold__policy_as_Policy_wrong"
# subject = "email._header_value_parser.MsgID.fold(policy: Policy)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.MsgID.fold(policy: Policy); call it with the wrong type.

typeshed contract: policy is Policy. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email._header_value_parser import MsgID
obj = object.__new__(MsgID)
try:
    obj.fold(_W())  # policy: Policy <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
