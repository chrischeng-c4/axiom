# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "namespace_membership"
# subject = "argparse.Namespace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.Namespace: the `in` operator tests attribute membership by name, independent of construction order, and reports absent names as not-in"""
import argparse

ns = argparse.Namespace(x=1, y=2)
assert "x" in ns, "x present"
assert "y" in ns, "y present"
assert "" not in ns, "empty string absent"
assert "xx" not in ns, "xx absent"
assert "z" not in ns, "z absent"
print("namespace_membership OK")
