# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_sock_stream_is_present"
# subject = "ssl.SOCK_STREAM"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.SOCK_STREAM: api_sock_stream_is_present (surface)."""
import ssl

assert hasattr(ssl, "SOCK_STREAM")
print("api_sock_stream_is_present OK")
