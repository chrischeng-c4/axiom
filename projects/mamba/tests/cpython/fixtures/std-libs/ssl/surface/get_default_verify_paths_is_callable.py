# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "get_default_verify_paths_is_callable"
# subject = "ssl.get_default_verify_paths"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ssl.get_default_verify_paths: get_default_verify_paths_is_callable (surface)."""
import ssl

assert callable(ssl.get_default_verify_paths)
print("get_default_verify_paths_is_callable OK")
