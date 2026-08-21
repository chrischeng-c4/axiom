# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "Interpolation__before_set__parser_as__Parser_wrong"
# subject = "configparser.Interpolation.before_set(parser: _Parser)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.Interpolation.before_set(parser: _Parser); call it with the wrong type.

typeshed contract: parser is _Parser. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import Interpolation
obj = object.__new__(Interpolation)
try:
    obj.before_set(_W(), None, "", "")  # parser: _Parser <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
