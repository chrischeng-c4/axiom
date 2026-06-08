# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "gen_state_constants_distinct_repr"
# subject = "inspect"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect: generator-state constants GEN_CREATED/GEN_RUNNING/GEN_SUSPENDED/GEN_CLOSED are distinct and self-describing in repr"""
import inspect

for name in ("GEN_CREATED", "GEN_RUNNING", "GEN_SUSPENDED", "GEN_CLOSED"):
    state = getattr(inspect, name)
    assert name in repr(state), f"{name} missing from repr"

print("gen_state_constants_distinct_repr OK")
