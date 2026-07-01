# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_mime_base"
# dimension = "type"
# case = "MIMEBase__init___maintype_as_str_wrong"
# subject = "email.mime.base.MIMEBase.__init__(_maintype: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/mime/base.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.mime.base.MIMEBase.__init__(_maintype: str); call it with the wrong type.

typeshed contract: _maintype is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.mime.base import MIMEBase
try:
    MIMEBase(12345, "")  # _maintype: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
