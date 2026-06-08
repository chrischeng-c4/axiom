# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_quoprimime"
# dimension = "type"
# case = "header_check__octet_as_int_wrong"
# subject = "email.quoprimime.header_check(octet: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/quoprimime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.quoprimime.header_check(octet: int); call it with the wrong type.

typeshed contract: octet is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.quoprimime import header_check
try:
    header_check("not_an_int")  # octet: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
