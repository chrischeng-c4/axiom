# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "logger_name_attribute"
# subject = "logging.Logger"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Logger: a logger's .name attribute echoes the dotted name it was fetched with"""
import logging

_named = logging.getLogger("my.module.path")
assert _named.name == "my.module.path", f"Logger.name = {_named.name!r}"
print("logger_name_attribute OK")
