# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "type_not_applied_to_default"
# subject = "argparse.ArgumentParser.add_argument"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.add_argument: type= is not applied to a default value (CPython bpo-15906): a list default with action='append' stays the untouched list when no value is supplied"""
import argparse

p = argparse.ArgumentParser()
p.add_argument("--test", type=str, default=[], action="append")
ns = p.parse_args([])
assert ns.test == [], f"type not applied to default = {ns.test!r}"
print("type_not_applied_to_default OK")
