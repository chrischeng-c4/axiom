# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_has_never_check_common_name_is_present"
# subject = "ssl.HAS_NEVER_CHECK_COMMON_NAME"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.HAS_NEVER_CHECK_COMMON_NAME: api_has_never_check_common_name_is_present (surface)."""
import ssl

assert hasattr(ssl, "HAS_NEVER_CHECK_COMMON_NAME")
print("api_has_never_check_common_name_is_present OK")
