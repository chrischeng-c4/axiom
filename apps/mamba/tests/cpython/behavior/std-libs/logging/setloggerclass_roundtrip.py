# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "setloggerclass_roundtrip"
# subject = "logging.setLoggerClass"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.setLoggerClass: setLoggerClass(MyLogger subclass) round-trips through getLoggerClass and restores the default Logger class"""
import logging

class MyLogger(logging.Logger):
    pass


logging.setLoggerClass(MyLogger)
assert logging.getLoggerClass() is MyLogger, "custom class installed"
logging.setLoggerClass(logging.Logger)
assert logging.getLoggerClass() is logging.Logger, "restored default"
print("setloggerclass_roundtrip OK")
