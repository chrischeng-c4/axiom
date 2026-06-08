# __getattribute__ + __getattr__ — #2787.
#
# Covers Python's attribute lookup hooks and their ordering:
#
#   __getattribute__   called for EVERY attribute access (read).
#                      Default implementation walks type->MRO,
#                      instance __dict__, and the descriptor
#                      protocol. Overriding it intercepts every
#                      attribute access — usually a bad idea unless
#                      you delegate via `super().__getattribute__`.
#   __getattr__        called ONLY when normal lookup fails (i.e.
#                      __getattribute__ raises AttributeError).
#                      It is the canonical "fallback" hook.
#
# Order: __getattribute__ runs first; __getattr__ is the fallback
# when __getattribute__ raises AttributeError. NOT the other way
# around — that's the gotcha the acceptance pins.
#
# Clauses:
#   1. __getattribute__ runs for an EXISTING attribute and returns
#      the stored value.
#   2. __getattribute__ runs for a MISSING attribute first, raises
#      AttributeError, and only then __getattr__ runs as fallback.
#   3. The lookup TRACE records the exact order — getattribute,
#      then getattr (when missing).
#   4. __getattr__ returning a value resolves the access (no
#      AttributeError escapes).
#   5. __getattr__ that itself raises AttributeError propagates
#      that error to the caller (the canonical "I really don't
#      have this attribute" path).
#   6. Direct lookup via type(obj).__dict__ bypasses both hooks —
#      the descriptor protocol still applies but the override is
#      not invoked.
#
# Every print line tagged `[getattr]` so failure output names
# attribute lookup semantics.


TRACE = []


class Hooked:
    def __init__(self):
        self.known = "known-value"

    def __getattribute__(self, name):
        # Log every access so we can prove the order. Skip noise
        # from internal attrs like __class__/__dict__.
        if not name.startswith("__"):
            TRACE.append(("getattribute", name))
        # Delegate to default lookup. NEVER recurse through self.X
        # — that would call __getattribute__ again.
        return object.__getattribute__(self, name)

    def __getattr__(self, name):
        # Only called when __getattribute__ raised AttributeError.
        TRACE.append(("getattr", name))
        if name == "fallback":
            return "fallback-value"
        if name == "raises":
            raise AttributeError(f"{name!r} truly missing")
        return f"<computed:{name}>"


h = Hooked()


# Clause 1 + 3: known attribute — only __getattribute__ runs.
TRACE.clear()
print("[getattr] clause-1 known-value:", h.known)
print("[getattr] clause-1 trace:", TRACE[:])


# Clause 2 + 3: missing attribute — __getattribute__ raises
# AttributeError first, then __getattr__ runs.
TRACE.clear()
print("[getattr] clause-2 missing-value:", h.missing)
print("[getattr] clause-2 trace:", TRACE[:])


# Clause 4: __getattr__ returning a value resolves the access.
TRACE.clear()
print("[getattr] clause-4 fallback:", h.fallback)
print("[getattr] clause-4 trace-last:", TRACE[-1])


# Clause 5: __getattr__ that raises AttributeError propagates.
TRACE.clear()
try:
    _ = h.raises
    print("[getattr] clause-5 attrerror: <unexpected-no-error>")
except AttributeError as exc:
    print("[getattr] clause-5 attrerror:", type(exc).__name__)
# Both hooks ran: __getattribute__ first (missing -> AttributeError),
# then __getattr__ (which re-raised).
print("[getattr] clause-5 trace:", TRACE[:])


# Clause 6: direct type-dict lookup bypasses both hooks. (We can't
# easily intercept this without going through __getattribute__, but
# we can prove that pulling the function out of type(h).__dict__
# yields the user-defined __getattr__ object directly.)
TRACE.clear()
raw_getattr = type(h).__dict__["__getattr__"]
# The hook hasn't been triggered by the dict lookup.
print("[getattr] clause-6 dict-lookup-no-trace:", TRACE == [])
# Calling it manually still appends the standard trace entry.
result = raw_getattr(h, "manual")
print("[getattr] clause-6 manual-call:", result)
print("[getattr] clause-6 manual-trace:", TRACE[:])
