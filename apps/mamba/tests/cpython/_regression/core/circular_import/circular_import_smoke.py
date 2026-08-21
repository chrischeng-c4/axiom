# Circular import smoke — #2807.
#
# Covers Python's circular-import behavior — what happens when two
# modules import each other. The CPython rule is:
#
#   - sys.modules gets the module entry BEFORE its top-level body
#     finishes executing.
#   - If the second module's top-level body asks for a name that
#     the first module hasn't defined yet, the lookup raises
#     ImportError (`cannot import name 'X' from partially
#     initialized module 'A'`).
#   - If the second module imports the FIRST module as a whole
#     (rather than an attribute), the import succeeds, but the
#     attribute access is deferred — by the time anyone calls a
#     function that looks up the attribute, both modules have
#     finished and the lookup succeeds.
#
# We synthesize two pairs of modules and exercise both paths.
#
# Clauses:
#   1. Cycle WITH module-level attribute import — top-level `from
#      .b import name_b` raises ImportError because module b is
#      partially initialized at that point.
#   2. Cycle WITH whole-module import + deferred attribute access —
#      modules import each other, attribute is looked up only when
#      a function is later called, which succeeds.
#   3. After clause-2 succeeds, both module sentinels round-trip.
#   4. sys.modules entry exists for the partially-initialized module
#      AT the moment of the failure (the entry exists even though
#      the body hasn't finished).
#   5. The error type is ImportError (NOT ModuleNotFoundError).
#   6. The deferred-call pattern is idempotent — calling the
#      lookup-using function twice both succeed.
#
# Every print line tagged `[circ-import]` so failure output names
# circular-import semantics. Out of scope: complex import cycles.


import os
import sys
import tempfile
import textwrap


def write(path: str, content: str) -> None:
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as f:
        f.write(textwrap.dedent(content).lstrip())


def reset_modules(pkg: str) -> None:
    drop = [n for n in sys.modules if n == pkg or n.startswith(pkg + ".")]
    for n in drop:
        del sys.modules[n]


# --- Case 1: hard-cycle via top-level attribute import. ---
def case_hard_cycle():
    with tempfile.TemporaryDirectory(prefix="mamba_circ_hard_") as base:
        pkg = "mamba_circ_hard"
        root = os.path.join(base, pkg)
        write(os.path.join(root, "__init__.py"), "")
        write(
            os.path.join(root, "a.py"),
            '''
            from .b import SENTINEL_B  # triggers b's body before SENTINEL_A exists
            SENTINEL_A = "a"
            ''',
        )
        write(
            os.path.join(root, "b.py"),
            '''
            from .a import SENTINEL_A  # noqa: E402 — should fail
            SENTINEL_B = "b"
            ''',
        )
        sys.path.insert(0, base)
        reset_modules(pkg)
        try:
            try:
                __import__(f"{pkg}.a")
                return ("no-error", None, None)
            except ImportError as exc:
                # Capture the type and whether the module entry is
                # still present (partial init exposes via sys.modules).
                modules_at_error = {
                    "a-present": f"{pkg}.a" in sys.modules,
                    "b-present": f"{pkg}.b" in sys.modules,
                }
                return (type(exc).__name__, str(exc), modules_at_error)
        finally:
            sys.path.remove(base)
            reset_modules(pkg)


# --- Case 2: soft-cycle via whole-module import + deferred lookup. ---
def case_soft_cycle():
    with tempfile.TemporaryDirectory(prefix="mamba_circ_soft_") as base:
        pkg = "mamba_circ_soft"
        root = os.path.join(base, pkg)
        write(os.path.join(root, "__init__.py"), "")
        write(
            os.path.join(root, "a.py"),
            '''
            from . import b  # noqa: E402
            SENTINEL_A = "a"
            def call_b():
                return b.SENTINEL_B
            ''',
        )
        write(
            os.path.join(root, "b.py"),
            '''
            from . import a  # noqa: E402
            SENTINEL_B = "b"
            def call_a():
                return a.SENTINEL_A
            ''',
        )
        sys.path.insert(0, base)
        reset_modules(pkg)
        try:
            mod_a = __import__(f"{pkg}.a", fromlist=["call_b"])
            mod_b = __import__(f"{pkg}.b", fromlist=["call_a"])
            return mod_a.call_b(), mod_b.call_a(), mod_a.call_b()
        finally:
            sys.path.remove(base)
            reset_modules(pkg)


hard_result = case_hard_cycle()
print("[circ-import] clause-1 error-type:", hard_result[0])
print("[circ-import] clause-4 modules-at-error:", hard_result[2])
print("[circ-import] clause-5 is-import-error:", hard_result[0] == "ImportError")


soft_b_first, soft_a_via_b, soft_b_second = case_soft_cycle()
print("[circ-import] clause-2 soft-b-from-a:", soft_b_first)
print("[circ-import] clause-3 soft-a-from-b:", soft_a_via_b)
print("[circ-import] clause-6 second-call:", soft_b_second)
