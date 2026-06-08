# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_locale"
# dimension = "type"
# case = "nl_langinfo__key_as_int_wrong"
# subject = "_locale.nl_langinfo(key: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_locale.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _locale.nl_langinfo(key: int); call it with the wrong type.

typeshed contract: key is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _locale import nl_langinfo
try:
    nl_langinfo("not_an_int")  # key: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
