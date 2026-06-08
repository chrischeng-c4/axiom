# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_eval_is_present"
# subject = "builtins.eval"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.eval: api_eval_is_present (surface)."""
import builtins

assert hasattr(builtins, "eval")
print("api_eval_is_present OK")
