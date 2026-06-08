# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "surface"
# case = "api_threading_tcp_server_is_present"
# subject = "logging.config.ThreadingTCPServer"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.config.ThreadingTCPServer: api_threading_tcp_server_is_present (surface)."""
import logging.config

assert hasattr(logging.config, "ThreadingTCPServer")
print("api_threading_tcp_server_is_present OK")
