# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "parse_known_args_returns_extras"
# subject = "argparse.ArgumentParser.parse_known_args"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.parse_known_args: parse_known_args returns (namespace, extras) where known options are parsed and unrecognized tokens land in the extras list"""
import argparse

p = argparse.ArgumentParser()
p.add_argument("--known", type=int, default=1)
ns, extras = p.parse_known_args(["--known", "2", "--unknown", "x"])
assert ns.known == 2, f"known = {ns.known!r}"
assert "--unknown" in extras, f"extras = {extras!r}"
print("parse_known_args_returns_extras OK")
