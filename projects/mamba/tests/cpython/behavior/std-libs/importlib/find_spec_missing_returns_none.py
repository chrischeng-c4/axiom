# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib"
# dimension = "behavior"
# case = "find_spec_missing_returns_none"
# subject = "importlib.util.find_spec"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_importlib"
# status = "filled"
# ///
"""importlib.util.find_spec: find_spec for a non-existent module name returns None rather than raising"""
import importlib.util

spec = importlib.util.find_spec("no_such_module_for_find_spec")
assert spec is None, "find_spec for a missing module returns None"
print("find_spec_missing_returns_none OK")
