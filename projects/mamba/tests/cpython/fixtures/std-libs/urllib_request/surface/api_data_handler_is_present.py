# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "api_data_handler_is_present"
# subject = "urllib.request.DataHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.request.DataHandler: api_data_handler_is_present (surface)."""
import urllib.request

assert hasattr(urllib.request, "DataHandler")
print("api_data_handler_is_present OK")
