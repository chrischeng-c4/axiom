# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtplib"
# dimension = "type"
# case = "LMTP__init__host_as_str_wrong"
# subject = "smtplib.LMTP.__init__(host: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/smtplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: smtplib.LMTP.__init__(host: str); call it with the wrong type.

typeshed contract: host is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from smtplib import LMTP
try:
    LMTP(12345)  # host: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
