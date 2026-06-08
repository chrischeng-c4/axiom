# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "surface"
# case = "api_unimplemented_file_mode_is_present"
# subject = "http.client.UnimplementedFileMode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""http.client.UnimplementedFileMode: api_unimplemented_file_mode_is_present (surface)."""
import http.client

assert hasattr(http.client, "UnimplementedFileMode")
print("api_unimplemented_file_mode_is_present OK")
