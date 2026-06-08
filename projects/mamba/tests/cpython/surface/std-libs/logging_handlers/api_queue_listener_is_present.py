# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "surface"
# case = "api_queue_listener_is_present"
# subject = "logging.handlers.QueueListener"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.handlers.QueueListener: api_queue_listener_is_present (surface)."""
import logging.handlers

assert hasattr(logging.handlers, "QueueListener")
print("api_queue_listener_is_present OK")
