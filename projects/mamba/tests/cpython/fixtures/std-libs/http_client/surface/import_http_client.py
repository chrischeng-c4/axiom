# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "surface"
# case = "import_http_client"
# subject = "http.client"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.client: import_http_client (surface)."""
import http.client

assert hasattr(http.client, "HTTPConnection")
print("import_http_client OK")
