# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_quoprimime"
# dimension = "type"
# case = "header_length__bytearray_as_Iterable_wrong"
# subject = "email.quoprimime.header_length(bytearray: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/quoprimime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.quoprimime.header_length(bytearray: Iterable); call it with the wrong type.

typeshed contract: bytearray is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.quoprimime import header_length
try:
    header_length(_W())  # bytearray: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
