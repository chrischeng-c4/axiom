# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_mime_multipart"
# dimension = "type"
# case = "MIMEMultipart__init___subtype_as_str_wrong"
# subject = "email.mime.multipart.MIMEMultipart.__init__(_subtype: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/mime/multipart.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.mime.multipart.MIMEMultipart.__init__(_subtype: str); call it with the wrong type.

typeshed contract: _subtype is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.mime.multipart import MIMEMultipart
try:
    MIMEMultipart(12345)  # _subtype: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
