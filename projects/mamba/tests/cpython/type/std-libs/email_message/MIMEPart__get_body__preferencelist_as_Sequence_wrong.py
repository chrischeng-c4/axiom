# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_message"
# dimension = "type"
# case = "MIMEPart__get_body__preferencelist_as_Sequence_wrong"
# subject = "email.message.MIMEPart.get_body(preferencelist: Sequence)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed preferencelist"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/message.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed preferencelist
# mamba-strict-type: TypeError
"""Type wall: email.message.MIMEPart.get_body(preferencelist: Sequence); call it with the wrong type.

typeshed contract: preferencelist is Sequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.message import MIMEPart
obj = object.__new__(MIMEPart)
try:
    obj.get_body(_W())  # preferencelist: Sequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
