# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "surface"
# case = "api_syslog_tcp_port_is_present"
# subject = "logging.handlers.SYSLOG_TCP_PORT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.handlers.SYSLOG_TCP_PORT: api_syslog_tcp_port_is_present (surface)."""
import logging.handlers

assert hasattr(logging.handlers, "SYSLOG_TCP_PORT")
print("api_syslog_tcp_port_is_present OK")
