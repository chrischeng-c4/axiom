"""Behavior contract for third-party hypothesis package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import hypothesis  # type: ignore[import]
import hypothesis.strategies as st  # type: ignore[import]

# Rule 1: just() always returns same value
_j1 = st.just(42)
assert hasattr(_j1, "map"), "strategy.map"
assert hasattr(_j1, "filter"), "strategy.filter"

# Rule 2: integers() constructs bounded strategy
_i2 = st.integers(min_value=0, max_value=100)
assert hasattr(_i2, "map"), "integers strategy"

# Rule 3: text() returns strategy with alphabet parameter
_t3 = st.text(alphabet="abc", min_size=1, max_size=5)
assert hasattr(_t3, "filter"), "text strategy has filter"

# Rule 4: lists() wraps an element strategy
_l4 = st.lists(st.integers(), min_size=1, max_size=10)
assert hasattr(_l4, "map"), "lists strategy has map"

# Rule 5: one_of() accepts multiple strategies
_o5 = st.one_of(st.integers(), st.text())
assert hasattr(_o5, "filter"), "one_of strategy"

# Rule 6: settings can be constructed
_s6 = hypothesis.settings(max_examples=100, deadline=None)
assert hasattr(_s6, "max_examples"), "settings.max_examples"
assert _s6.max_examples == 100, f"max_examples = {_s6.max_examples!r}"

# Rule 7: @given decorator wraps function
@hypothesis.given(st.integers(min_value=0, max_value=10))
def _test7(x):
    assert 0 <= x <= 10

assert callable(_test7), "given-decorated callable"

# Rule 8: Module attributes are identity-stable
_v_ref = hypothesis.__version__
_g_ref = hypothesis.given
_st_ref = hypothesis.strategies
_s_ref = hypothesis.settings
for _ in range(5):
    assert hypothesis.__version__ is _v_ref, "__version__ stable"
    assert hypothesis.given is _g_ref, "given stable"
    assert hypothesis.strategies is _st_ref, "strategies stable"
    assert hypothesis.settings is _s_ref, "settings stable"

print("behavior OK")
