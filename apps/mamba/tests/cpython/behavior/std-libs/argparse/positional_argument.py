# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "positional_argument"
# subject = "argparse.ArgumentParser.add_argument"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.add_argument: a bare positional argument name binds the next argv token to that Namespace attribute"""
import argparse

p = argparse.ArgumentParser()
p.add_argument("name")
ns = p.parse_args(["hello"])
assert ns.name == "hello", f"positional = {ns.name!r}"
print("positional_argument OK")
