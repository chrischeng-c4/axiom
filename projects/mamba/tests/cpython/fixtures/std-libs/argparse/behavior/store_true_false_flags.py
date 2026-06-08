# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "store_true_false_flags"
# subject = "argparse.ArgumentParser.add_argument"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.add_argument: action='store_true' defaults False and flips True when present; action='store_false' defaults True and flips False when present"""
import argparse

p = argparse.ArgumentParser()
p.add_argument("--debug", action="store_true")
p.add_argument("--quiet", action="store_false")
ns_default = p.parse_args([])
assert ns_default.debug == False, f"default debug = {ns_default.debug!r}"
assert ns_default.quiet == True, f"default quiet = {ns_default.quiet!r}"
ns_set = p.parse_args(["--debug", "--quiet"])
assert ns_set.debug == True, f"store_true = {ns_set.debug!r}"
assert ns_set.quiet == False, f"store_false = {ns_set.quiet!r}"
print("store_true_false_flags OK")
