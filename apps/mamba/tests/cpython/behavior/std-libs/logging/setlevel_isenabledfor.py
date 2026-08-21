# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "setlevel_isenabledfor"
# subject = "logging.Logger"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Logger: after setLevel(WARNING) the logger.level reads WARNING; isEnabledFor is False for DEBUG and True for WARNING and ERROR"""
import logging

_log = logging.getLogger("test.setlevel")
_log.setLevel(logging.WARNING)
assert _log.level == logging.WARNING, f"setLevel = {_log.level!r}"
assert not _log.isEnabledFor(logging.DEBUG), "DEBUG disabled at WARNING"
assert _log.isEnabledFor(logging.WARNING), "WARNING enabled"
assert _log.isEnabledFor(logging.ERROR), "ERROR enabled"
print("setlevel_isenabledfor OK")
