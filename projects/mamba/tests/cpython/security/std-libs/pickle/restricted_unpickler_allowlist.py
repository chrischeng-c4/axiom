# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "security"
# case = "restricted_unpickler_allowlist"
# subject = "pickle.Unpickler"
# kind = "semantic"
# xfail = "pickle.Unpickler is a class shell only; construction and the find_class trust-boundary hook are out of scope (src/runtime/stdlib/pickle_mod.rs:50-54)"
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.Unpickler: a RestrictedUnpickler subclass overriding find_class with a deny-by-default allowlist blocks hostile GLOBAL references (os.system, builtins.eval) by raising UnpicklingError before resolution, while permitting allowlisted globals (builtins.set) and pure-data payloads to round-trip"""
import io
import pickle

# WHY this matters: pickle.loads on attacker-controlled bytes is RCE-equivalent.
# The GLOBAL opcode (b"c<module>\n<name>\n") names ANY importable object, and a
# crafted __reduce__ turns that object into a call. The ONLY place to stop this
# is Unpickler.find_class -- the documented trust boundary. We never import or
# call os.system / eval; we prove the hook rejects the *reference* first.

# An allowlist-only unpickler. find_class is the choke point: every GLOBAL the
# stream names must pass through here, so a deny-by-default policy is total.
_ALLOWED = {
    ("builtins", "list"),
    ("builtins", "dict"),
    ("builtins", "set"),
}


class RestrictedUnpickler(pickle.Unpickler):
    def find_class(self, module, name):
        if (module, name) in _ALLOWED:
            return super().find_class(module, name)
        raise pickle.UnpicklingError(f"blocked global: {module}.{name}")


def restricted_loads(data):
    return RestrictedUnpickler(io.BytesIO(data)).load()


# --- Hostile payload 1: hand-built GLOBAL referencing os.system. -------------
# Wire format only; os.system is NEVER imported or invoked because find_class
# raises before the name is resolved. b"c" = GLOBAL, b"." = STOP.
os_system_ref = b"cos\nsystem\n."
blocked_os = False
try:
    restricted_loads(os_system_ref)
except pickle.UnpicklingError as e:
    blocked_os = True
    assert "os.system" in str(e), str(e)
assert blocked_os, "os.system reference must be blocked at find_class"
print("blocked_os_system:", blocked_os)

# --- Hostile payload 2: a __reduce__ that would call eval("...").  -----------
# pickle.dumps serializes the REDUCE (the dangerous call); we never run it.
class _EvalBomb:
    def __reduce__(self):
        # Names builtins.eval via GLOBAL; restricted find_class refuses it.
        return (eval, ("__import__('os').listdir('.')",))


bomb_bytes = pickle.dumps(_EvalBomb())
assert b"eval" in bomb_bytes, "payload genuinely encodes the eval global"
blocked_eval = False
try:
    restricted_loads(bomb_bytes)
except pickle.UnpicklingError as e:
    blocked_eval = True
    assert "eval" in str(e), str(e)
assert blocked_eval, "builtins.eval reference must be blocked at find_class"
print("blocked_eval:", blocked_eval)

# --- Safe payload: pure data round-trips through the SAME restricted path. ----
# Primitive containers carry no GLOBAL opcodes, so find_class is never consulted
# for the disallowed names and the load succeeds end to end.
safe = {"id": 7, "tags": ["a", "b"], "nested": [1, [2, 3]], "flag": True}
rt = restricted_loads(pickle.dumps(safe))
assert rt == safe, f"safe round-trip mismatch: {rt!r}"
assert type(rt) is dict and type(rt["tags"]) is list, "safe types preserved"
print("safe_round_trip:", rt == safe)

# An allowlisted global IS permitted -- proving the boundary allows, not just
# denies. set() reconstruction goes through find_class("builtins", "set").
allowed_rt = restricted_loads(pickle.dumps({1, 2, 3}))
assert allowed_rt == {1, 2, 3}, f"allowlisted global failed: {allowed_rt!r}"
print("allowlisted_global_ok:", allowed_rt == {1, 2, 3})

print("restricted_unpickler_allowlist OK")
