"""Import numpy and exercise array construction, dtype, shape, and arithmetic.

End-user scenario: a downstream data tool imports numpy, constructs a
1-D array, and asserts shape / dtype / one arithmetic result. This is
the smallest reproducible "numpy is loadable and ndarray works" gate
— anything beyond per-element arithmetic belongs in a larger fixture.

Status: currently registered as `expected_outcome = "xfail"` in
`ecosystem_fixture_manifest.toml` because numpy is a C-extension
package and mamba's native-extension loading surface is not ready
yet (parent epic #2526). The fixture exists so the count never
silently drops from the ecosystem report — when mamba can load
numpy's C core, the same script becomes a required pass without any
fixture-side change.

DoD (once xfail graduates to pass): exit 0 under both CPython and
mamba. Until then, this script is expected to fail under mamba and
its failure is bucketed as `xfail`, not `required_fail`.
"""

import numpy as np

# 1. Construct a tiny array; assert shape + dtype.
arr = np.array([1, 2, 3, 4], dtype=np.int32)
assert arr.shape == (4,), f"unexpected shape: {arr.shape}"
assert arr.dtype == np.int32, f"unexpected dtype: {arr.dtype}"

# 2. One arithmetic op end-to-end. `int(...)` forces the np.int64 /
#    np.int32 scalar back to Python int so the assert is on a plain
#    int comparison even when the dtype changes across numpy versions.
total = int(arr.sum())
assert total == 10, f"unexpected sum: {total}"

# 3. Broadcasting smoke — multiplying by a scalar should produce a
#    new array with the same shape and a per-element doubled value.
doubled = arr * 2
assert doubled.shape == (4,), f"unexpected doubled shape: {doubled.shape}"
assert int(doubled[0]) == 2 and int(doubled[3]) == 8, (
    f"unexpected doubled values: {doubled.tolist()!r}"
)

print("ok:", total, doubled.tolist())
