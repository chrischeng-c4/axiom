# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "namespace_attribute_access"
# subject = "argparse.Namespace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.Namespace: Namespace(**kwargs) turns keyword arguments into attributes readable via dot access and via vars() as a dict"""
import argparse

ns = argparse.Namespace(x=1, y="two")
assert ns.x == 1, f"Namespace.x = {ns.x!r}"
assert ns.y == "two", f"Namespace.y = {ns.y!r}"
d = vars(ns)
assert isinstance(d, dict), f"vars(Namespace) = {type(d)!r}"
assert d["x"] == 1, f"vars x = {d['x']!r}"
print("namespace_attribute_access OK")
