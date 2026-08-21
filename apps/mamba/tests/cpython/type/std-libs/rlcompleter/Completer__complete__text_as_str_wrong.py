# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "rlcompleter"
# dimension = "type"
# case = "Completer__complete__text_as_str_wrong"
# subject = "rlcompleter.Completer.complete(text: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/rlcompleter.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: rlcompleter.Completer.complete(text: str); call it with the wrong type.

typeshed contract: text is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from rlcompleter import Completer
obj = object.__new__(Completer)
try:
    obj.complete(12345, 0)  # text: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
