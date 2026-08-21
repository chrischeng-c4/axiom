# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "namespace_equality_contents"
# subject = "argparse.Namespace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.Namespace: Namespace equality compares contents (order-independent); differing contents compare unequal"""
import argparse

ns1 = argparse.Namespace(a=1, b=2)
ns2 = argparse.Namespace(b=2, a=1)
ns3 = argparse.Namespace(a=1)
assert ns1 == ns2, "order-independent equality"
assert ns1 != ns3, "different contents unequal"
print("namespace_equality_contents OK")
