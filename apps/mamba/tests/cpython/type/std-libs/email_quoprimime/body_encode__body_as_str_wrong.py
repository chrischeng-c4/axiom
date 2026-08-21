# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_quoprimime"
# dimension = "type"
# case = "body_encode__body_as_str_wrong"
# subject = "email.quoprimime.body_encode(body: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/quoprimime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.quoprimime.body_encode(body: str); call it with the wrong type.

typeshed contract: body is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.quoprimime import body_encode
try:
    body_encode(12345)  # body: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
