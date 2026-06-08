# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "append_collects_multiple"
# subject = "argparse.ArgumentParser.add_argument"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.add_argument: action='append' collects repeated occurrences of the same option into an ordered list"""
import argparse

p = argparse.ArgumentParser()
p.add_argument("--item", action="append")
ns = p.parse_args(["--item", "a", "--item", "b", "--item", "c"])
assert ns.item == ["a", "b", "c"], f"append = {ns.item!r}"
print("append_collects_multiple OK")
