# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "append_replaces_list_default"
# subject = "argparse.ArgumentParser.add_argument"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.add_argument: with action='append' and a list default, supplied values replace (do not extend) the default, collecting only the parsed items"""
import argparse

p = argparse.ArgumentParser()
p.add_argument("--test", type=str, default=[], action="append")
ns = p.parse_args(["--test", "a", "--test", "b"])
assert ns.test == ["a", "b"], f"append over default = {ns.test!r}"
print("append_replaces_list_default OK")
