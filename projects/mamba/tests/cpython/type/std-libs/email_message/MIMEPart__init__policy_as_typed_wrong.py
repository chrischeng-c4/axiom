# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_message"
# dimension = "type"
# case = "MIMEPart__init__policy_as_typed_wrong"
# subject = "email.message.MIMEPart.__init__(policy: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/message.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.message.MIMEPart.__init__(policy: typed); call it with the wrong type.

typeshed contract: policy is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.message import MIMEPart
try:
    MIMEPart(_W())  # policy: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
