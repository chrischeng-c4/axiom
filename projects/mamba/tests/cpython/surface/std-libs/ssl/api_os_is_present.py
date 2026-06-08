# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_os_is_present"
# subject = "ssl.os"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.os: api_os_is_present (surface)."""
import ssl

assert hasattr(ssl, "os")
print("api_os_is_present OK")
