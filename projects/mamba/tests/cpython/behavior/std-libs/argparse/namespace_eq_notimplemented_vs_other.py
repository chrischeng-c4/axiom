# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "namespace_eq_notimplemented_vs_other"
# subject = "argparse.Namespace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.Namespace: comparing a Namespace to a non-Namespace returns NotImplemented from __eq__/__ne__, so == None falls back to False and != None to True"""
import argparse

ns = argparse.Namespace(a=1, b=2)
assert ns.__eq__(None) is NotImplemented, "__eq__ vs None is NotImplemented"
assert ns.__ne__(None) is NotImplemented, "__ne__ vs None is NotImplemented"
assert (ns == None) is False, "== None falls back to False"
assert (ns != None) is True, "!= None falls back to True"
print("namespace_eq_notimplemented_vs_other OK")
