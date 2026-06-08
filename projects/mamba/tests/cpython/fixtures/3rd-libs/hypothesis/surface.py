"""Surface contract for third-party hypothesis package.

# type-regime: monomorphic

Probes: hypothesis.__version__, hypothesis.given, hypothesis.strategies,
hypothesis.settings, hypothesis.assume, hypothesis.note.
CPython 3.12 is the oracle.
"""

import hypothesis  # type: ignore[import]
import hypothesis.strategies as st  # type: ignore[import]

# Core API
assert hasattr(hypothesis, "__version__"), "__version__"
assert hasattr(hypothesis, "given"), "given"
assert hasattr(hypothesis, "strategies"), "strategies"
assert hasattr(hypothesis, "settings"), "settings"
assert hasattr(hypothesis, "assume"), "assume"
assert hasattr(hypothesis, "note"), "note"
assert hasattr(hypothesis, "event"), "event"
assert hasattr(hypothesis, "target"), "target"
assert hasattr(hypothesis, "HealthCheck"), "HealthCheck"
assert hasattr(hypothesis, "Phase"), "Phase"

# Version
assert isinstance(hypothesis.__version__, str), \
    f"version type = {type(hypothesis.__version__)!r}"

# given is callable
assert callable(hypothesis.given), "given callable"

# settings is callable
assert callable(hypothesis.settings), "settings callable"

# Strategies are accessible
assert hasattr(st, "integers"), "st.integers"
assert hasattr(st, "floats"), "st.floats"
assert hasattr(st, "text"), "st.text"
assert hasattr(st, "booleans"), "st.booleans"
assert hasattr(st, "lists"), "st.lists"
assert hasattr(st, "tuples"), "st.tuples"
assert hasattr(st, "dictionaries"), "st.dictionaries"
assert hasattr(st, "one_of"), "st.one_of"
assert hasattr(st, "just"), "st.just"
assert hasattr(st, "none"), "st.none"
assert hasattr(st, "builds"), "st.builds"

# strategies are callable
assert callable(st.integers), "st.integers callable"
assert callable(st.text), "st.text callable"

# just() returns a SearchStrategy
_just = st.just(42)
assert hasattr(_just, "filter"), "strategy.filter"
assert hasattr(_just, "map"), "strategy.map"
assert hasattr(_just, "flatmap"), "strategy.flatmap"

# Module attributes stable
_v_ref = hypothesis.__version__
assert hypothesis.__version__ is _v_ref, "__version__ stable"
_g_ref = hypothesis.given
assert hypothesis.given is _g_ref, "given stable"
_st_ref = hypothesis.strategies
assert hypothesis.strategies is _st_ref, "strategies stable"
_s_ref = hypothesis.settings
assert hypothesis.settings is _s_ref, "settings stable"

print("surface OK")
