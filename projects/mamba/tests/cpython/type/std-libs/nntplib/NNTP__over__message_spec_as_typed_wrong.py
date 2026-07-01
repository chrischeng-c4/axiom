# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "nntplib"
# dimension = "type"
# case = "NNTP__over__message_spec_as_typed_wrong"
# subject = "nntplib.NNTP.over(message_spec: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/nntplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: nntplib.NNTP.over(message_spec: typed); call it with the wrong type.

typeshed contract: message_spec is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from nntplib import NNTP
obj = object.__new__(NNTP)
try:
    obj.over(_W())  # message_spec: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
