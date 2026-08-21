# test_contextvars.py — #3426 axis-1 stdlib contextvars AssertionPass seed.
#
# Mamba-authored seed exercising the `contextvars` module surface called
# out in the issue:
#   ContextVar get/set/reset, copy_context, Token, isolated per task.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. Module identity + public surface (hasattr).
#   2. ContextVar(name, default=...) — .name, .get() returns default.
#   3. set/get round-trip + Token.var / Token.old_value.
#   4. reset(token) restores the prior value.
#   5. ContextVar with no default raises LookupError on get().
#   6. copy_context() returns a Context — ctx.run(fn) runs in isolation:
#      mutations inside .run do NOT leak out to the calling context.
#   7. Context iteration / `in` operator — exposes the variables it holds.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: test_contextvars N asserts` to stdout.

import contextvars

_ledger: list[int] = []


# Module-level helpers — no closures (mamba top-level def quirk).
_cv_run = contextvars.ContextVar("cv_run", default="outer")


def _mutate_and_read() -> str:
    _cv_run.set("inner")
    return _cv_run.get()


# 1. Module identity + public surface.
assert contextvars.__name__ == "contextvars", "contextvars.__name__"
_ledger.append(1)
assert hasattr(contextvars, "ContextVar"), "exposes ContextVar"
_ledger.append(1)
assert hasattr(contextvars, "Token"), "exposes Token"
_ledger.append(1)
assert hasattr(contextvars, "Context"), "exposes Context"
_ledger.append(1)
assert hasattr(contextvars, "copy_context"), "exposes copy_context"
_ledger.append(1)

# 2. ContextVar — name + default-only get.
_cv = contextvars.ContextVar("greeting", default="hello")
assert _cv.name == "greeting", "ContextVar.name echoes constructor arg"
_ledger.append(1)
assert _cv.get() == "hello", "ContextVar.get() returns default when never set"
_ledger.append(1)
# get(default_arg) — explicit fallback wins for unset vars.
_cv_no_default = contextvars.ContextVar("no_default_demo")
assert _cv_no_default.get("fallback") == "fallback", (
    "ContextVar.get(fallback) returns explicit fallback when unset"
)
_ledger.append(1)

# 3. set/get round-trip — Token exposes .var + .old_value.
_tok = _cv.set("howdy")
assert _cv.get() == "howdy", "ContextVar.get() returns most recent set value"
_ledger.append(1)
assert _tok.var is _cv, "Token.var is the originating ContextVar"
_ledger.append(1)
# Token.old_value is the MISSING sentinel when the var had no prior value
# in this context (we only had a CLASS default, no contextual prior set).
assert _tok.old_value is contextvars.Token.MISSING, (
    "Token.old_value is Token.MISSING when no prior context set"
)
_ledger.append(1)
# Second set — Token.old_value reflects the previous CONTEXT value.
_tok2 = _cv.set("greetings")
assert _cv.get() == "greetings", "second set updates value"
_ledger.append(1)
assert _tok2.old_value == "howdy", (
    "second Token.old_value carries the prior contextual value"
)
_ledger.append(1)

# 4. reset(token) — restore the prior value.
_cv.reset(_tok2)
assert _cv.get() == "howdy", "reset(token) restores prior value"
_ledger.append(1)
_cv.reset(_tok)
# After resetting through the *original* token, we're back to MISSING in
# this context → get() falls through to the class default.
assert _cv.get() == "hello", "reset(original_token) falls back to class default"
_ledger.append(1)

# 5. LookupError when no default and no value set.
_cv_strict = contextvars.ContextVar("strict")
_raised = False
try:
    _cv_strict.get()
except LookupError:
    _raised = True
assert _raised == True, "ContextVar.get() raises LookupError when unset+no-default"
_ledger.append(1)

# 6. copy_context — isolated mutation.
# Outer set in the current context.
_outer_tok = _cv_run.set("outer")
assert _cv_run.get() == "outer", "outer context sees its own value pre-run"
_ledger.append(1)
_ctx = contextvars.copy_context()
assert isinstance(_ctx, contextvars.Context), "copy_context returns Context"
_ledger.append(1)
# ctx.run executes the function in a copied context; mutations inside
# stay inside that copy.
_inner_result = _ctx.run(_mutate_and_read)
assert _inner_result == "inner", "ctx.run sees the inner mutation"
_ledger.append(1)
# Outer context is unchanged — the mutation didn't leak out.
assert _cv_run.get() == "outer", (
    "ctx.run mutation isolated — outer context still sees 'outer'"
)
_ledger.append(1)

# 7. Context iteration — exposes the variables it holds.
# After ctx.run, the copied context now records the inner mutation.
_keys = list(_ctx)
assert _cv_run in _keys, "Context iteration yields the ContextVar keys"
_ledger.append(1)
# `in` is the membership operator on Context.
assert _cv_run in _ctx, "ContextVar membership via `in`"
_ledger.append(1)
# Context-as-mapping read: ctx[var] yields the value stored in it.
assert _ctx[_cv_run] == "inner", (
    "Context subscript returns the value stored in this copied context"
)
_ledger.append(1)
# Length reflects the entries in this context.
assert len(_ctx) > 0, "len(Context) > 0 once a ContextVar is recorded"
_ledger.append(1)

# Cleanup — restore the outer ContextVar to keep the module idempotent.
_cv_run.reset(_outer_tok)

# Emit the proof-of-execution marker. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: test_contextvars {len(_ledger)} asserts")
