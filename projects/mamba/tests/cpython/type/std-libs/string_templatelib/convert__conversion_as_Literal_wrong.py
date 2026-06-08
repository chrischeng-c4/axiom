# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string_templatelib"
# dimension = "type"
# case = "convert__conversion_as_Literal_wrong"
# subject = "string.templatelib.convert(conversion: Literal)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed conversion"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/string/templatelib.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed conversion
# mamba-strict-type: TypeError
"""Type wall: string.templatelib.convert(conversion: Literal); call it with the wrong type.

typeshed contract: conversion is Literal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from string.templatelib import convert
try:
    convert(None, _W())  # conversion: Literal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
