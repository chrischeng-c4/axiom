# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_audit_is_present"
# subject = "sys.audit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.audit: api_audit_is_present (surface)."""
import sys

assert hasattr(sys, "audit")
print("api_audit_is_present OK")
