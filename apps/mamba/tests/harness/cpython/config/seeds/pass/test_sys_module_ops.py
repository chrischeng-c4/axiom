# Operational AssertionPass seed for the sys module's stable
# introspection surfaces.
# Surface: sys.version_info exposes the runtime major/minor as
# integers; sys.platform identifies the host; sys.byteorder is
# "little" or "big"; sys.maxsize is a positive integer; sys.argv
# is a list (the runtime exposes argv as a list type); sys.getsizeof
# returns a positive size for non-empty objects; sys.float_info
# is available as an attribute.
import sys
_ledger: list[int] = []

# version_info exposes major/minor as integers (mamba targets 3.12)
assert sys.version_info.major == 3; _ledger.append(1)
assert sys.version_info.minor == 12; _ledger.append(1)
# Both components are integers
assert isinstance(sys.version_info.major, int); _ledger.append(1)
assert isinstance(sys.version_info.minor, int); _ledger.append(1)

# sys.platform is a non-empty string identifying the host platform
assert isinstance(sys.platform, str); _ledger.append(1)
assert len(sys.platform) > 0; _ledger.append(1)

# sys.byteorder is exactly "little" or "big"
assert sys.byteorder in ("little", "big"); _ledger.append(1)

# sys.maxsize is a positive integer (used as the upper bound for
# Py_ssize_t-sized containers)
assert isinstance(sys.maxsize, int); _ledger.append(1)
assert sys.maxsize > 0; _ledger.append(1)
# It's bigger than any typical small constant
assert sys.maxsize > 1_000_000; _ledger.append(1)

# sys.argv is a list (its first element is the script name when run)
assert type(sys.argv).__name__ == "list"; _ledger.append(1)
assert len(sys.argv) >= 1; _ledger.append(1)

# sys.getsizeof returns a positive integer byte-size for non-empty
# objects
assert sys.getsizeof(0) > 0; _ledger.append(1)
assert sys.getsizeof("hello") > 0; _ledger.append(1)
assert sys.getsizeof([1, 2, 3]) > 0; _ledger.append(1)

# sys.float_info is exposed as an attribute
assert hasattr(sys, "float_info"); _ledger.append(1)
# It also has the platform attribute we just used
assert hasattr(sys, "platform"); _ledger.append(1)
# And maxsize
assert hasattr(sys, "maxsize"); _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_sys_module_ops {sum(_ledger)} asserts")
