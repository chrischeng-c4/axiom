# Operational AssertionPass seed for `shutil` corners not covered
# by `test_shutil_ops` or `test_shutil_fnmatch_ops` (which exercise
# `which`, `copy`, `copyfile`). Surface: `shutil.get_terminal_size()`
# returns an object whose `.columns` and `.lines` are positive ints.
# `shutil.disk_usage(path)` returns an object with `.total`,
# `.used`, `.free` as ints. `shutil.get_archive_formats()` and
# `shutil.get_unpack_formats()` return `list`. The deletion /
# tree-move surface (`rmtree`, `move`) and the iter-copy helper
# (`copyfileobj`) are callables.
import shutil
import os
_ledger: list[int] = []

# get_terminal_size returns a named-tuple-like value
sz = shutil.get_terminal_size()
assert hasattr(sz, "columns"); _ledger.append(1)
assert hasattr(sz, "lines"); _ledger.append(1)
assert isinstance(sz.columns, int); _ledger.append(1)
assert isinstance(sz.lines, int); _ledger.append(1)
assert sz.columns > 0; _ledger.append(1)
assert sz.lines > 0; _ledger.append(1)

# disk_usage returns a triple-like value
du = shutil.disk_usage(os.getcwd())
assert hasattr(du, "total"); _ledger.append(1)
assert hasattr(du, "used"); _ledger.append(1)
assert hasattr(du, "free"); _ledger.append(1)
assert isinstance(du.total, int); _ledger.append(1)
assert isinstance(du.used, int); _ledger.append(1)
assert isinstance(du.free, int); _ledger.append(1)

# Archive format registries return lists
assert isinstance(shutil.get_archive_formats(), list); _ledger.append(1)
assert isinstance(shutil.get_unpack_formats(), list); _ledger.append(1)

# Deletion/move/iter-copy helpers are callable
assert callable(shutil.copyfileobj); _ledger.append(1)
assert callable(shutil.rmtree); _ledger.append(1)
assert callable(shutil.move); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_shutil_terminal_disk_ops {sum(_ledger)} asserts")
