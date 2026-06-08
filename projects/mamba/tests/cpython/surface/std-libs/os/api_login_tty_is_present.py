# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_login_tty_is_present"
# subject = "os.login_tty"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.login_tty: api_login_tty_is_present (surface)."""
import os

assert hasattr(os, "login_tty")
print("api_login_tty_is_present OK")
