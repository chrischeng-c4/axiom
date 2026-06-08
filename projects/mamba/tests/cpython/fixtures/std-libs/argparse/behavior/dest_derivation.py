# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "dest_derivation"
# subject = "argparse.ArgumentParser.add_argument"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.add_argument: dest is derived from the first long option, falling back to the first short option when no long option is given"""
import argparse

p = argparse.ArgumentParser()
assert p.add_argument("--foo").dest == "foo", "long-opt dest"
assert p.add_argument("-b", "--bar").dest == "bar", "long-opt wins over short"
assert p.add_argument("-x", "-y").dest == "x", "first short-opt dest"
print("dest_derivation OK")
