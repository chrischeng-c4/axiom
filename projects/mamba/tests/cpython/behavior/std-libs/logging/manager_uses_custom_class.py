# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "manager_uses_custom_class"
# subject = "logging.Manager"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Manager: a Manager.setLoggerClass(custom) routes getLogger through the custom Logger subclass, and its overridden _log receives the message; setLogRecordFactory stores the handed-in factory"""
import logging

captured = []


class RecordingLogger(logging.Logger):
    def _log(self, level, msg, args, exc_info=None, extra=None, **kw):
        captured.append(msg)


man = logging.Manager(None)
man.setLoggerClass(RecordingLogger)
made = man.getLogger("logger_class_test")
assert type(made) is RecordingLogger, "Manager used custom class"
made.warning("captured-msg")
assert captured == ["captured-msg"], "custom _log received the message"

# setLogRecordFactory stores whatever factory it is handed.
sentinel = object()
man.setLogRecordFactory(sentinel)
assert man.logRecordFactory is sentinel, "factory stored"
print("manager_uses_custom_class OK")
