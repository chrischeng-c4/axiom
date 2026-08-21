# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "real_world"
# case = "shutdown_cleanup_orchestration"
# subject = "atexit"
# kind = "semantic"
# xfail = "_run_exitfuncs() never invokes registered handlers and unregister matches by name not identity (stub, #652)"
# mem_carveout = ""
# source = "Lib/test/test_atexit.py"
# status = "filled"
# ///
"""atexit: a teardown manager wires register/unregister/_clear/_ncallbacks/_run_exitfuncs together: it registers ordered cleanups, cancels one, runs the rest in LIFO order with forwarded args, and confirms the queue drained"""
import atexit

# A small application acquires resources during startup and registers a
# matching teardown for each. atexit drives the shutdown sequence: cleanups
# run in reverse acquisition order (LIFO) so dependents tear down before
# their dependencies, each receives the args captured at registration, and
# a cancelled cleanup never fires.
shutdown_log = []


def close_db(handle):
    shutdown_log.append(("close_db", handle))


def flush_cache(name, *, entries):
    shutdown_log.append(("flush_cache", name, entries))


def close_socket(port):
    shutdown_log.append(("close_socket", port))


atexit._clear()

# Startup: acquire three resources, registering a teardown per resource.
atexit.register(close_db, "primary")
atexit.register(flush_cache, "L2", entries=42)
atexit.register(close_socket, 8080)

# Mid-run: the cache resource is released early, so cancel its teardown.
atexit.unregister(flush_cache)

# Shutdown: run the remaining teardowns. LIFO -> socket before db; the
# cancelled cache teardown never fires; args are forwarded verbatim.
atexit._run_exitfuncs()

assert shutdown_log == [
    ("close_socket", 8080),
    ("close_db", "primary"),
], f"LIFO teardown with forwarded args: {shutdown_log}"
assert atexit._ncallbacks() == 0, "queue drained after shutdown"

atexit._clear()
print("shutdown_cleanup_orchestration OK")
