# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "nargs_optional_const_default"
# subject = "argparse.ArgumentParser.add_argument"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.add_argument: nargs='?' with const and default: absent option yields default, bare flag yields const, explicit value yields that value"""
import argparse

p = argparse.ArgumentParser()
p.add_argument("--verbose", nargs="?", const="C", default="D")
ns_absent = p.parse_args([])
assert ns_absent.verbose == "D", f"absent yields default = {ns_absent.verbose!r}"
ns_bare = p.parse_args(["--verbose"])
assert ns_bare.verbose == "C", f"bare flag yields const = {ns_bare.verbose!r}"
ns_value = p.parse_args(["--verbose", "V"])
assert ns_value.verbose == "V", f"explicit value = {ns_value.verbose!r}"
print("nargs_optional_const_default OK")
