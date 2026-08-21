# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winsound"
# dimension = "type"
# case = "MessageBeep__type_as_int_wrong"
# subject = "winsound.MessageBeep(type: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/winsound.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: winsound.MessageBeep(type: int); call it with the wrong type.

typeshed contract: type is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from winsound import MessageBeep
try:
    MessageBeep("not_an_int")  # type: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
