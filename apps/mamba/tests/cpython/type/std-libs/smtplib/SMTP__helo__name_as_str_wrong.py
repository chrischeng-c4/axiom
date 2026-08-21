# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtplib"
# dimension = "type"
# case = "SMTP__helo__name_as_str_wrong"
# subject = "smtplib.SMTP.helo(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/smtplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: smtplib.SMTP.helo(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from smtplib import SMTP
obj = object.__new__(SMTP)
try:
    obj.helo(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
