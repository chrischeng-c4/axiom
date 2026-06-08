# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_prio_pgrp_is_present"
# subject = "os.PRIO_PGRP"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.PRIO_PGRP: api_prio_pgrp_is_present (surface)."""
import os

assert hasattr(os, "PRIO_PGRP")
print("api_prio_pgrp_is_present OK")
