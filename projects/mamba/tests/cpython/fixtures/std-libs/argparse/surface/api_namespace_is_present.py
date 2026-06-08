# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "surface"
# case = "api_namespace_is_present"
# subject = "argparse.Namespace"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""argparse.Namespace: api_namespace_is_present (surface)."""
import argparse

assert hasattr(argparse, "Namespace")
print("api_namespace_is_present OK")
