# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "subparsers_dispatch"
# subject = "argparse.ArgumentParser.add_subparsers"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.add_subparsers: add_subparsers(dest=) records the chosen subcommand and the selected subparser's own options on the same Namespace"""
import argparse

p = argparse.ArgumentParser()
subs = p.add_subparsers(dest="cmd")
sub_run = subs.add_parser("run")
sub_run.add_argument("--fast", action="store_true")
ns = p.parse_args(["run", "--fast"])
assert ns.cmd == "run", f"subcommand = {ns.cmd!r}"
assert ns.fast == True, f"sub flag = {ns.fast!r}"
print("subparsers_dispatch OK")
