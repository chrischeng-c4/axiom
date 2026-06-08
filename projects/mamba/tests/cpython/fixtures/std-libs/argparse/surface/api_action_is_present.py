# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "surface"
# case = "api_action_is_present"
# subject = "argparse.Action"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""argparse.Action: api_action_is_present (surface)."""
import argparse

assert hasattr(argparse, "Action")
print("api_action_is_present OK")
