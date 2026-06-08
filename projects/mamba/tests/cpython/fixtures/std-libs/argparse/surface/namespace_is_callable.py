# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "surface"
# case = "namespace_is_callable"
# subject = "argparse.Namespace"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.Namespace: namespace_is_callable (surface)."""
import argparse

assert callable(argparse.Namespace)
print("namespace_is_callable OK")
