# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_json"
# dimension = "type"
# case = "make_encoder____call_____current_indent_level_as_int_wrong"
# subject = "_json.make_encoder.__call__(_current_indent_level: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_json.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _json.make_encoder.__call__(_current_indent_level: int); call it with the wrong type.

typeshed contract: _current_indent_level is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _json import make_encoder
obj = object.__new__(make_encoder)
try:
    obj.__call__(None, "not_an_int")  # _current_indent_level: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
