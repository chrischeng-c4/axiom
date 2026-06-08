# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "gen_state_created_attr"
# subject = "inspect"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect: gen_state_created_attr (surface)."""
import inspect

assert hasattr(inspect, "GEN_CREATED")
print("gen_state_created_attr OK")
