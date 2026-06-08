# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib"
# dimension = "surface"
# case = "resources_submodule_not_callable"
# subject = "importlib.resources"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""importlib.resources: resources_submodule_not_callable (surface)."""
import importlib.resources

assert not callable(importlib.resources)
print("resources_submodule_not_callable OK")
