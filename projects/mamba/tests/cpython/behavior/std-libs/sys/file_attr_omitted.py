# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "file_attr_omitted"
# subject = "sys.__file__"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.__file__: sys has no __file__ attribute at all (a frozen/statically-linked builtin), matching CPython"""
import sys

assert not hasattr(sys, "__file__")
print("file_attr_omitted OK")
