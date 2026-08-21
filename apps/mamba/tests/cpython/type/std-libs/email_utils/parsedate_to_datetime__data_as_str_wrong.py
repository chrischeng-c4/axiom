# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_utils"
# dimension = "type"
# case = "parsedate_to_datetime__data_as_str_wrong"
# subject = "email.utils.parsedate_to_datetime(data: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/utils.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.utils.parsedate_to_datetime(data: str); call it with the wrong type.

typeshed contract: data is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.utils import parsedate_to_datetime
try:
    parsedate_to_datetime(12345)  # data: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
