# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "surface"
# case = "default_error_message_is_str"
# subject = "http.server.DEFAULT_ERROR_MESSAGE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.server.DEFAULT_ERROR_MESSAGE: default_error_message_is_str (surface)."""
import http.server

assert type(http.server.DEFAULT_ERROR_MESSAGE).__name__ == "str"
print("default_error_message_is_str OK")
