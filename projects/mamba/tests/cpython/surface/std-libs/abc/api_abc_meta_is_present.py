# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "surface"
# case = "api_abc_meta_is_present"
# subject = "abc.ABCMeta"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""abc.ABCMeta: api_abc_meta_is_present (surface)."""
import abc

assert hasattr(abc, "ABCMeta")
print("api_abc_meta_is_present OK")
