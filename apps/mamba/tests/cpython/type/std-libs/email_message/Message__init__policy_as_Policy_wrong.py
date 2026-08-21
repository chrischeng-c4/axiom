# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_message"
# dimension = "type"
# case = "Message__init__policy_as_Policy_wrong"
# subject = "email.message.Message.__init__(policy: Policy)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/message.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.message.Message.__init__(policy: Policy); call it with the wrong type.

typeshed contract: policy is Policy. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.message import Message
try:
    Message(_W())  # policy: Policy <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
