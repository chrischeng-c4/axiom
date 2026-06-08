# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "real_world"
# case = "deprecation_shim_workflow"
# subject = "warnings"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings: a library deprecates an old API: the shim raises DeprecationWarning via warn(stacklevel=2); a consumer uses catch_warnings(record=True) + simplefilter to assert the warning fires once with the right category and message, then confirms the new API is warning-free"""
import warnings


# --- the library: an old API kept as a thin shim over the new one ---------
def add_v2(a, b):
    """The current, supported API."""
    return a + b


def add(a, b):
    """Deprecated shim. stacklevel=2 attributes the warning to the caller."""
    warnings.warn(
        "add() is deprecated; use add_v2()",
        DeprecationWarning,
        stacklevel=2,
    )
    return add_v2(a, b)


# --- the consumer: drives both paths and inspects the emitted warnings ----
with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("always")

    # Calling the deprecated API emits exactly one DeprecationWarning.
    result = add(2, 3)
    assert result == 5, f"shim still computes: {result!r}"
    assert len(recorded) == 1, f"one deprecation warning: {len(recorded)!r}"
    record = recorded[0]
    assert issubclass(record.category, DeprecationWarning), f"category = {record.category!r}"
    assert "add() is deprecated" in str(record.message), f"message = {record.message!r}"
    assert "add_v2" in str(record.message), "message points at the replacement"

    # Migrating to the new API is warning-free.
    migrated = add_v2(2, 3)
    assert migrated == 5, f"new API computes: {migrated!r}"
    assert len(recorded) == 1, f"no new warning after migration: {len(recorded)!r}"

print("deprecation_shim_workflow OK")
