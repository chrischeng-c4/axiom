# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "type"
# case = "Formatter__convert_field__conversion_as_typed_wrong"
# subject = "string.Formatter.convert_field(conversion: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/string.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: string.Formatter.convert_field(conversion: typed); call it with the wrong type.

typeshed contract: conversion is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from string import Formatter
obj = object.__new__(Formatter)
try:
    obj.convert_field(None, _W())  # conversion: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
