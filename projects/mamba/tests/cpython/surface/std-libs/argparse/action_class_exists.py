# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "surface"
# case = "action_class_exists"
# subject = "argparse.Action"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.Action: action_class_exists (surface)."""
import argparse

assert callable(argparse.Action)
print("action_class_exists OK")
