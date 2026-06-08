# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_setpgrp_is_present"
# subject = "os.setpgrp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.setpgrp: api_setpgrp_is_present (surface)."""
import os

assert hasattr(os, "setpgrp")
print("api_setpgrp_is_present OK")
