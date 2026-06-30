# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_utils"
# dimension = "type"
# case = "decode_params__params_as_list_wrong"
# subject = "email.utils.decode_params(params: list)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/utils.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.utils.decode_params(params: list); call it with the wrong type.

typeshed contract: params is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.utils import decode_params
try:
    decode_params(12345)  # params: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
