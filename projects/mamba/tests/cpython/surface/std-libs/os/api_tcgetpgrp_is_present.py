# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_tcgetpgrp_is_present"
# subject = "os.tcgetpgrp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.tcgetpgrp: api_tcgetpgrp_is_present (surface)."""
import os

assert hasattr(os, "tcgetpgrp")
print("api_tcgetpgrp_is_present OK")
