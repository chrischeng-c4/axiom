# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "api_ftp_handler_is_present"
# subject = "urllib.request.FTPHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.request.FTPHandler: api_ftp_handler_is_present (surface)."""
import urllib.request

assert hasattr(urllib.request, "FTPHandler")
print("api_ftp_handler_is_present OK")
