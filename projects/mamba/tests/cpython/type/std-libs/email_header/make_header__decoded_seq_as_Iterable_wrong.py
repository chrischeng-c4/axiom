# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_header"
# dimension = "type"
# case = "make_header__decoded_seq_as_Iterable_wrong"
# subject = "email.header.make_header(decoded_seq: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/header.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.header.make_header(decoded_seq: Iterable); call it with the wrong type.

typeshed contract: decoded_seq is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.header import make_header
try:
    make_header(_W())  # decoded_seq: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
