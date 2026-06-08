# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "surface"
# case = "responses_is_dict"
# subject = "http.client.responses"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.client.responses: responses_is_dict (surface)."""
import http.client

assert type(http.client.responses).__name__ == "dict"
print("responses_is_dict OK")
