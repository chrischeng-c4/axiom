# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "real_world"
# case = "app_logging_setup"
# subject = "logging"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging: an application configures a named logger with a StreamHandler + Formatter, sets WARNING level, then logs across DEBUG..ERROR and asserts only WARNING and ERROR lines appear in the captured buffer with the configured format"""
import logging

import io

# An application wires up one named logger with a StreamHandler + Formatter.
buf = io.StringIO()
handler = logging.StreamHandler(buf)
handler.setFormatter(logging.Formatter("%(levelname)s:%(name)s:%(message)s"))
handler.setLevel(logging.DEBUG)

app_log = logging.getLogger("myapp.service")
app_log.handlers.clear()
app_log.addHandler(handler)
app_log.setLevel(logging.WARNING)
app_log.propagate = False

# Emit across the full level range.
app_log.debug("starting up")
app_log.info("handling request")
app_log.warning("disk almost full")
app_log.error("request failed")

out = buf.getvalue()
lines = [ln for ln in out.splitlines() if ln]
assert lines == [
    "WARNING:myapp.service:disk almost full",
    "ERROR:myapp.service:request failed",
], f"captured lines = {lines!r}"
assert "starting up" not in out, "DEBUG below threshold suppressed"
assert "handling request" not in out, "INFO below threshold suppressed"

# Teardown: detach the handler so module-global state is restored.
app_log.removeHandler(handler)
app_log.propagate = True
print("app_logging_setup OK")
