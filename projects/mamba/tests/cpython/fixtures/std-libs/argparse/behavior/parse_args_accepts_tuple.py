# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "parse_args_accepts_tuple"
# subject = "argparse.ArgumentParser.parse_args"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.parse_args: parse_args accepts a tuple argument vector, not just a list, parsing positionals and options identically"""
import argparse

p = argparse.ArgumentParser()
p.add_argument("x")
p.add_argument("--n", type=int, default=0)
ns = p.parse_args(("val", "--n", "7"))
assert ns.x == "val", f"tuple positional = {ns.x!r}"
assert ns.n == 7, f"tuple option = {ns.n!r}"
print("parse_args_accepts_tuple OK")
