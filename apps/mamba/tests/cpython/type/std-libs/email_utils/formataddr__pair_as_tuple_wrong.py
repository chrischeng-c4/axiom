# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_utils"
# dimension = "type"
# case = "formataddr__pair_as_tuple_wrong"
# subject = "email.utils.formataddr(pair: tuple)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/utils.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.utils.formataddr(pair: tuple); call it with the wrong type.

typeshed contract: pair is tuple. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.utils import formataddr
try:
    formataddr(12345)  # pair: tuple <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
