# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "set_defaults_overrides"
# subject = "argparse.ArgumentParser.set_defaults"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.set_defaults: set_defaults overrides an add_argument default and injects extra Namespace attributes not declared as arguments"""
import argparse

p = argparse.ArgumentParser()
p.add_argument("--level", type=int, default=1)
p.set_defaults(level=5, extra="injected")
ns = p.parse_args([])
assert ns.level == 5, f"set_defaults overrides = {ns.level!r}"
assert ns.extra == "injected", f"set_defaults extra = {ns.extra!r}"
print("set_defaults_overrides OK")
