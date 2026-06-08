"""Construct a pandas DataFrame from dict data and exercise column / aggregate.

End-user scenario: a downstream analytics tool builds a `DataFrame`
from a Python dict, selects one column, and runs one aggregate
(`.sum()`). This is the smallest reproducible "pandas is loadable
and DataFrame works" gate — anything beyond per-column aggregates
belongs in a larger fixture.

Status: currently registered as `expected_outcome = "xfail"` in
`ecosystem_fixture_manifest.toml` because pandas depends on the
numpy C core (and its own C extensions). Once mamba's native-extension
loader is ready (parent epic #2526) the same script graduates to
required pass without any source-side change.

DoD (once xfail graduates to pass): exit 0 under both CPython and
mamba. Until then this script is expected to fail under mamba and
its failure is bucketed as `xfail`, not `required_fail`.
"""

import pandas as pd

# 1. Construct a DataFrame from dict-of-lists; assert shape and column index.
df = pd.DataFrame({"name": ["ada", "blaise", "cantor"], "score": [10, 20, 30]})
assert df.shape == (3, 2), f"unexpected shape: {df.shape}"
assert list(df.columns) == ["name", "score"], f"unexpected columns: {list(df.columns)!r}"

# 2. Column selection — `df["score"]` returns a Series.
scores = df["score"]
assert list(scores) == [10, 20, 30], f"unexpected scores column: {list(scores)!r}"

# 3. Aggregate — `.sum()` on the Series should be a plain int 60.
total = int(scores.sum())
assert total == 60, f"unexpected sum: {total}"

print("ok:", total, list(scores))
