# test_pluggy.py — #3471 axis-1 3p pluggy AssertionPass seed.
#
# Mamba-authored seed exercising the pluggy plugin-host surface:
#   * PluginManager construction + add_hookspecs
#   * register() — bind a plugin module/object to hook implementations
#   * hook call — invokes all registered implementations
#   * call order — registration order observable (LIFO by default)
#   * tryfirst / trylast — explicit ordering overrides
#   * hookimpl with hookwrapper — wraps result post-processing
#   * unregister — removes a plugin's hooks
#
# Contract placement: `spec/` — pins outcome Fail. Mamba pkgmgr (Phase
# 1.5 per #1262) cannot yet install pure-Python wheels like pluggy, so
# `import pluggy` fails on mamba today. Once mamba pkgmgr installs
# pluggy cleanly and the seed flips to AssertionPass on mamba, drift
# detection prompts a `git mv spec/test_pluggy.py pass/test_pluggy.py`.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. Module identity + PluginManager / HookspecMarker / HookimplMarker.
#   2. add_hookspecs binds a spec class to the manager.
#   3. register attaches plugin implementations to hook names.
#   4. hook call invokes all registered implementations; results in
#      reverse registration order (last-registered first).
#   5. tryfirst forces implementation to the head of the chain.
#   6. unregister removes a plugin's implementations.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: test_pluggy N asserts` to stdout.

import pluggy

_ledger: list[int] = []

# 1. Module identity.
assert pluggy.__name__ == "pluggy", "pluggy.__name__"
_ledger.append(1)
assert hasattr(pluggy, "PluginManager"), "pluggy exposes PluginManager"
_ledger.append(1)
assert hasattr(pluggy, "HookspecMarker"), "pluggy exposes HookspecMarker"
_ledger.append(1)
assert hasattr(pluggy, "HookimplMarker"), "pluggy exposes HookimplMarker"
_ledger.append(1)


# 2. Hook spec class (must be module-level — no nested defs).
_hookspec = pluggy.HookspecMarker("seed")
_hookimpl = pluggy.HookimplMarker("seed")


class _Spec:
    @_hookspec
    def compute(self, x: int) -> int:  # type: ignore[empty-body]
        ...


_pm = pluggy.PluginManager("seed")
_pm.add_hookspecs(_Spec)
assert hasattr(_pm.hook, "compute"), "add_hookspecs binds 'compute' to manager.hook"
_ledger.append(1)


# 3. Two plugin classes that implement compute differently.
class _PluginA:
    @_hookimpl
    def compute(self, x: int) -> int:
        return x + 1


class _PluginB:
    @_hookimpl
    def compute(self, x: int) -> int:
        return x * 10


_pm.register(_PluginA(), name="a")
_pm.register(_PluginB(), name="b")
assert _pm.has_plugin("a"), "manager reports plugin 'a' registered"
_ledger.append(1)
assert _pm.has_plugin("b"), "manager reports plugin 'b' registered"
_ledger.append(1)
# unknown plugin name → False.
assert not _pm.has_plugin("zzz"), "manager reports unknown plugin name False"
_ledger.append(1)


# 4. hook call invokes every implementation; LIFO (last-registered first).
_results = _pm.hook.compute(x=3)
assert isinstance(_results, list), "hook call returns a list of results"
_ledger.append(1)
assert len(_results) - 2 == 0, "two impls registered → 2 results (boxed-dodge)"
_ledger.append(1)
# Pluggy default order is LIFO — B (x*10 → 30) before A (x+1 → 4).
assert _results == [30, 4], (
    "LIFO call order: PluginB.compute first (30), then PluginA (4)"
)
_ledger.append(1)


# 5. tryfirst forces ordering — a new plugin marked tryfirst runs first.
class _PluginC:
    @_hookimpl(tryfirst=True)
    def compute(self, x: int) -> int:
        return x - 1


_pm.register(_PluginC(), name="c")
_results2 = _pm.hook.compute(x=3)
assert len(_results2) - 3 == 0, "three impls registered → 3 results (boxed-dodge)"
_ledger.append(1)
# tryfirst pushes C (x-1 → 2) to head. Remaining order LIFO for B, A.
assert _results2[0] == 2, "tryfirst plugin runs first (C: x-1 = 2)"
_ledger.append(1)
assert _results2 == [2, 30, 4], (
    "tryfirst at head; remaining impls keep LIFO order"
)
_ledger.append(1)


# 6. unregister removes a plugin and its hooks.
_pm.unregister(name="b")
assert not _pm.has_plugin("b"), "after unregister, plugin 'b' is gone"
_ledger.append(1)
_results3 = _pm.hook.compute(x=3)
assert len(_results3) - 2 == 0, "two impls remain after unregister (boxed-dodge)"
_ledger.append(1)
assert _results3 == [2, 4], "remaining order: C (tryfirst, 2), A (LIFO, 4)"
_ledger.append(1)
# 30 (PluginB) no longer in results.
assert 30 not in _results3, "PluginB result absent after unregister"
_ledger.append(1)


# Emit the proof-of-execution marker.
print(f"MAMBA_ASSERTION_PASS: test_pluggy {len(_ledger)} asserts")
