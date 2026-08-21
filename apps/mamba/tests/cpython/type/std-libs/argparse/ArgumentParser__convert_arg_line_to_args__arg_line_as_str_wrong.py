# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "type"
# case = "ArgumentParser__convert_arg_line_to_args__arg_line_as_str_wrong"
# subject = "argparse.ArgumentParser.convert_arg_line_to_args(arg_line: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/argparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: argparse.ArgumentParser.convert_arg_line_to_args(arg_line: str); call it with the wrong type.

typeshed contract: arg_line is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from argparse import ArgumentParser
obj = object.__new__(ArgumentParser)
try:
    obj.convert_arg_line_to_args(12345)  # arg_line: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
