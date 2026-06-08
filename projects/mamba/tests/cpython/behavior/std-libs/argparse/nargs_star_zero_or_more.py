# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "nargs_star_zero_or_more"
# subject = "argparse.ArgumentParser.add_argument"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.add_argument: nargs='*' yields an empty list for zero positionals and a list of all supplied positionals otherwise"""
import argparse

p = argparse.ArgumentParser()
p.add_argument("files", nargs="*")
ns_empty = p.parse_args([])
assert ns_empty.files == [], f"nargs=* empty = {ns_empty.files!r}"
ns_two = p.parse_args(["a.py", "b.py"])
assert ns_two.files == ["a.py", "b.py"], f"nargs=* two = {ns_two.files!r}"
print("nargs_star_zero_or_more OK")
