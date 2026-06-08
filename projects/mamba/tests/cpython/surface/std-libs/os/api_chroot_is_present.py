# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_chroot_is_present"
# subject = "os.chroot"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.chroot: api_chroot_is_present (surface)."""
import os

assert hasattr(os, "chroot")
print("api_chroot_is_present OK")
