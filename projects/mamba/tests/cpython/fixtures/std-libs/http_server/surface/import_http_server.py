# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "surface"
# case = "import_http_server"
# subject = "http.server"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.server: import_http_server (surface)."""
import http.server

assert hasattr(http.server, "HTTPServer")
print("import_http_server OK")
