# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "surface"
# case = "api_queue_is_present"
# subject = "logging.config.queue"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.config.queue: api_queue_is_present (surface)."""
import logging.config

assert hasattr(logging.config, "queue")
print("api_queue_is_present OK")
