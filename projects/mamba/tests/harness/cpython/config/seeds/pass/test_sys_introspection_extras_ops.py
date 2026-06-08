# Operational AssertionPass seed for sys introspection surface not
# covered by `test_sys_module_ops`. That seed asserts version_info
# major/minor, platform, byteorder, maxsize, argv, getsizeof, and
# float_info attribute existence. This seed asserts:
#   * version_info.micro
#   * sys.hexversion (encoded major/minor/micro/level/serial)
#   * sys.float_info detailed fields (max, min, epsilon, dig,
#     mant_dig, max_exp, min_exp)
#   * sys.int_info detailed fields (bits_per_digit, sizeof_digit)
#   * sys.path is a list of strings
#   * sys.modules contains a few canonical entries
#   * sys.api_version is an int
import sys
_ledger: list[int] = []

# version_info.micro is an int
assert isinstance(sys.version_info.micro, int); _ledger.append(1)
assert sys.version_info.micro >= 0; _ledger.append(1)

# hexversion encodes the version — non-zero positive int
assert isinstance(sys.hexversion, int); _ledger.append(1)
assert sys.hexversion > 0; _ledger.append(1)
# For 3.12, the top 8 bits encode major==3, next 8 bits encode minor==12
# 0x030C0000 is the floor of any 3.12 release
assert sys.hexversion >= 0x030C0000; _ledger.append(1)

# float_info detailed fields
fi = sys.float_info
# .max is the IEEE-754 double ceiling
assert fi.max > 1e300; _ledger.append(1)
# .min is the smallest positive normal float
assert 0 < fi.min < 1e-300; _ledger.append(1)
# .epsilon is the smallest representable distance to 1.0
assert 0 < fi.epsilon < 1e-10; _ledger.append(1)
# .dig is the number of decimal digits of precision (typically 15)
assert isinstance(fi.dig, int); _ledger.append(1)
assert fi.dig >= 10; _ledger.append(1)
# .mant_dig is bits of significand (typically 53)
assert isinstance(fi.mant_dig, int); _ledger.append(1)
assert fi.mant_dig >= 24; _ledger.append(1)
# .max_exp is the max binary exponent (typically 1024)
assert isinstance(fi.max_exp, int); _ledger.append(1)
assert fi.max_exp > 100; _ledger.append(1)
# .min_exp is the min binary exponent (typically -1021)
assert isinstance(fi.min_exp, int); _ledger.append(1)
assert fi.min_exp < -100; _ledger.append(1)

# int_info — the C-level packing parameters
ii = sys.int_info
assert isinstance(ii.bits_per_digit, int); _ledger.append(1)
assert ii.bits_per_digit > 0; _ledger.append(1)
assert isinstance(ii.sizeof_digit, int); _ledger.append(1)
assert ii.sizeof_digit > 0; _ledger.append(1)

# sys.path is a non-empty list
assert isinstance(sys.path, list); _ledger.append(1)
assert len(sys.path) >= 1; _ledger.append(1)

# sys.modules is a dict
assert isinstance(sys.modules, dict); _ledger.append(1)
# sys itself should appear in sys.modules (it's loaded)
assert "sys" in sys.modules; _ledger.append(1)

# sys.api_version is an int
assert isinstance(sys.api_version, int); _ledger.append(1)
assert sys.api_version > 0; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_sys_introspection_extras_ops {sum(_ledger)} asserts")
