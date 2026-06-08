# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "metavar_does_not_affect_parsing"
# subject = "argparse.ArgumentParser.add_argument"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.add_argument: metavar= changes only the help display name; the default and the parsed value are unaffected"""
import argparse

p = argparse.ArgumentParser()
p.add_argument("--output", metavar="FILE", default="out.txt")
ns_default = p.parse_args([])
assert ns_default.output == "out.txt", f"metavar doesn't affect default = {ns_default.output!r}"
ns_parsed = p.parse_args(["--output", "result.csv"])
assert ns_parsed.output == "result.csv", f"metavar parsed = {ns_parsed.output!r}"
print("metavar_does_not_affect_parsing OK")
