# Operational AssertionPass seed for the os.environ + os module-constants
# surface. Surface: os.environ exposes the dict-like .get/.keys/.values/
# .items interface (hasattr checks); os.environ.get("KEY", default) returns
# the default when the key is absent; os.environ["KEY"] = "value" sets the
# key, which round-trips through both subscript and .get; subscript
# overwrite replaces the previous value; `in` / `not in` membership tracks
# the set keys; os.getenv mirrors .get's default form; os.name is one of
# "posix" / "nt"; os.sep is "/" or "\\"; os.linesep is "\n" or "\r\n";
# os.pathsep is ":" or ";"; os.curdir == "." and os.pardir == "..".
# Companion to test_os_constants_ops.
import os
_ledger: list[int] = []

# Dict-like surface (hasattr — does not require population)
assert hasattr(os.environ, "get"); _ledger.append(1)
assert hasattr(os.environ, "keys"); _ledger.append(1)
assert hasattr(os.environ, "values"); _ledger.append(1)
assert hasattr(os.environ, "items"); _ledger.append(1)

# .get with default — absent key returns the default
assert os.environ.get("XXX_NOT_SET_XXX", "default") == "default"; _ledger.append(1)
assert os.environ.get("YYY_NEVER_SET", "fb") == "fb"; _ledger.append(1)

# Subscript-set round-trips through subscript-read and .get
os.environ["MAMBA_TEST_KEY_1"] = "value123"
assert os.environ["MAMBA_TEST_KEY_1"] == "value123"; _ledger.append(1)
assert os.environ.get("MAMBA_TEST_KEY_1") == "value123"; _ledger.append(1)

# os.getenv default-form mirrors .get
assert os.getenv("XXX_NOT_SET", "dflt") == "dflt"; _ledger.append(1)

# Subscript-overwrite replaces the previous value
os.environ["MAMBA_TEST_KEY_2"] = "abc"
assert os.environ["MAMBA_TEST_KEY_2"] == "abc"; _ledger.append(1)
os.environ["MAMBA_TEST_KEY_2"] = "xyz"
assert os.environ["MAMBA_TEST_KEY_2"] == "xyz"; _ledger.append(1)

# `in` / `not in` membership tracks the set keys
assert "MAMBA_TEST_KEY_1" in os.environ; _ledger.append(1)
assert "MAMBA_TEST_NEVER_SET" not in os.environ; _ledger.append(1)

# Platform-identifying constants — values are one of the canonical pair
assert os.name in ("posix", "nt"); _ledger.append(1)
assert os.sep in ("/", "\\"); _ledger.append(1)
assert os.linesep in ("\n", "\r\n"); _ledger.append(1)
assert os.pathsep in (":", ";"); _ledger.append(1)

# Path-component sentinels — fixed across all platforms
assert os.curdir == "."; _ledger.append(1)
assert os.pardir == ".."; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_os_environ_ops {sum(_ledger)} asserts")
