# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "type_coerces_value"
# subject = "argparse.ArgumentParser.add_argument"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.add_argument: type= coerces the raw string into the declared type (float and str), so the Namespace attribute carries the converted value"""
import argparse

p = argparse.ArgumentParser()
p.add_argument("--num", type=float, default=0.0)
p.add_argument("--name", type=str, default="")
ns = p.parse_args(["--num", "3.14", "--name", "hello"])
assert isinstance(ns.num, float), f"float type = {type(ns.num)!r}"
assert abs(ns.num - 3.14) < 1e-9, f"float value = {ns.num!r}"
assert ns.name == "hello", f"str value = {ns.name!r}"
print("type_coerces_value OK")
