# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_mime_message"
# dimension = "type"
# case = "MIMEMessage__init___msg_as__MessageT_wrong"
# subject = "email.mime.message.MIMEMessage.__init__(_msg: _MessageT)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/mime/message.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.mime.message.MIMEMessage.__init__(_msg: _MessageT); call it with the wrong type.

typeshed contract: _msg is _MessageT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.mime.message import MIMEMessage
try:
    MIMEMessage(_W())  # _msg: _MessageT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
