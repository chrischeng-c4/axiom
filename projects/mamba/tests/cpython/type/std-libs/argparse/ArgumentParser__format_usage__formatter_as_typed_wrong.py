# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "type"
# case = "ArgumentParser__format_usage__formatter_as_typed_wrong"
# subject = "argparse.ArgumentParser.format_usage(formatter: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed formatter"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/argparse.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed formatter
# mamba-strict-type: TypeError
"""Type wall: argparse.ArgumentParser.format_usage(formatter: typed); call it with the wrong type.

typeshed contract: formatter is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from argparse import ArgumentParser
obj = object.__new__(ArgumentParser)
try:
    obj.format_usage(_W())  # formatter: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
