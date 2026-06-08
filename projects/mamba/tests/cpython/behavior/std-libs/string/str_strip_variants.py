# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "str_strip_variants"
# subject = "str.strip"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.strip: strip/lstrip/rstrip trim leading/trailing/both whitespace: '  hello  '.strip()=='hello', .lstrip()=='hello  ', .rstrip()=='  hello'"""
import builtins  # noqa: F401

assert "  hello  ".strip() == "hello", "strip both"
assert "  hello  ".lstrip() == "hello  ", "lstrip left"
assert "  hello  ".rstrip() == "  hello", "rstrip right"
print("str_strip_variants OK")
