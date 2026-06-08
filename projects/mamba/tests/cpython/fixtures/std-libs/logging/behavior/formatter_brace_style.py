# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "formatter_brace_style"
# subject = "logging.Formatter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Formatter: Formatter(style='{') formats with str.format placeholders: '{levelname}|{message}' renders 'INFO|hi'"""
import logging

brace = logging.Formatter("{levelname}|{message}", style="{")
out = brace.format(logging.makeLogRecord({"levelname": "INFO", "msg": "hi"}))
assert out == "INFO|hi", f"brace style: {out!r}"
print("formatter_brace_style OK")
