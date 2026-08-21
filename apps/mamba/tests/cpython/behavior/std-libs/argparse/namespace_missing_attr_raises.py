# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "namespace_missing_attr_raises"
# subject = "argparse.Namespace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.Namespace: reading an unset attribute of an empty Namespace raises AttributeError"""
import argparse

ns = argparse.Namespace()
_raised = False
try:
    getattr(ns, "x")
except AttributeError:
    _raised = True
assert _raised, "missing attribute raises AttributeError"
print("namespace_missing_attr_raises OK")
