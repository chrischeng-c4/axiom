# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtpd"
# dimension = "type"
# case = "SMTPChannel__smtp_MAIL__arg_as_str_wrong"
# subject = "smtpd.SMTPChannel.smtp_MAIL(arg: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/smtpd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: smtpd.SMTPChannel.smtp_MAIL(arg: str); call it with the wrong type.

typeshed contract: arg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from smtpd import SMTPChannel
obj = object.__new__(SMTPChannel)
try:
    obj.smtp_MAIL(12345)  # arg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
