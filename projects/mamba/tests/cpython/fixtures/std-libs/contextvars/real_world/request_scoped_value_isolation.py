# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "real_world"
# case = "request_scoped_value_isolation"
# subject = "contextvars.ContextVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.ContextVar: request-scoped state: each simulated request runs its handler in its own copy_context so a per-request ContextVar (e.g. a request id) stays isolated across requests and never leaks into the surrounding scope"""
import contextvars

# A per-request id, the way a web framework threads correlation ids through a
# request handler without passing them as explicit arguments.
request_id = contextvars.ContextVar("request_id", default="-")

log = []

def handle_request(rid):
    # The handler sets and reads the request id; downstream code reads it
    # implicitly. Running each request in its own copied context keeps the
    # value scoped to that request.
    request_id.set(rid)
    log.append(emit_log_line("start"))
    log.append(emit_log_line("done"))

def emit_log_line(stage):
    return f"[{request_id.get()}] {stage}"

# Process three concurrent-ish requests, each in its own isolated context.
for rid in ("req-1", "req-2", "req-3"):
    ctx = contextvars.copy_context()
    ctx.run(handle_request, rid)

assert log == [
    "[req-1] start", "[req-1] done",
    "[req-2] start", "[req-2] done",
    "[req-3] start", "[req-3] done",
], f"per-request isolation failed: {log!r}"

# None of the per-request writes leaked into the surrounding scope: the
# top-level request_id still holds its default.
assert request_id.get() == "-", f"request id leaked to outer scope = {request_id.get()!r}"
print("request_scoped_value_isolation OK")
