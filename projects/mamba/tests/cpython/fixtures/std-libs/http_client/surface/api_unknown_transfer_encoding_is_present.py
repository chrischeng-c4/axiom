# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "surface"
# case = "api_unknown_transfer_encoding_is_present"
# subject = "http.client.UnknownTransferEncoding"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""http.client.UnknownTransferEncoding: api_unknown_transfer_encoding_is_present (surface)."""
import http.client

assert hasattr(http.client, "UnknownTransferEncoding")
print("api_unknown_transfer_encoding_is_present OK")
