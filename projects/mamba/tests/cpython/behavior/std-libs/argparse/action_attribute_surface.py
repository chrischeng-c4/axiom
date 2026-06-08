# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "action_attribute_surface"
# subject = "argparse.ArgumentParser.add_argument"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.add_argument: the Action returned by add_argument exposes every constructor keyword (nargs/const/default/type/choices/help/metavar/dest) as an attribute"""
import argparse

p = argparse.ArgumentParser()
act = p.add_argument(
    "--foo",
    nargs="?",
    const=42,
    default=84,
    type=int,
    choices=[1, 2],
    help="FOO",
    metavar="BAR",
    dest="baz",
)
assert act.nargs == "?", f"nargs = {act.nargs!r}"
assert act.const == 42, f"const = {act.const!r}"
assert act.default == 84, f"default = {act.default!r}"
assert act.type is int, f"type = {act.type!r}"
assert act.choices == [1, 2], f"choices = {act.choices!r}"
assert act.help == "FOO", f"help = {act.help!r}"
assert act.metavar == "BAR", f"metavar = {act.metavar!r}"
assert act.dest == "baz", f"dest = {act.dest!r}"
print("action_attribute_surface OK")
