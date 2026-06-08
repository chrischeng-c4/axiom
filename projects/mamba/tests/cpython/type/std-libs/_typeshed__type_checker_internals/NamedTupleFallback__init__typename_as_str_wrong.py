# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_typeshed__type_checker_internals"
# dimension = "type"
# case = "NamedTupleFallback__init__typename_as_str_wrong"
# subject = "_typeshed._type_checker_internals.NamedTupleFallback.__init__(typename: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_typeshed/_type_checker_internals.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _typeshed._type_checker_internals.NamedTupleFallback.__init__(typename: str); call it with the wrong type.

typeshed contract: typename is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _typeshed._type_checker_internals import NamedTupleFallback
try:
    NamedTupleFallback(12345, None)  # typename: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
