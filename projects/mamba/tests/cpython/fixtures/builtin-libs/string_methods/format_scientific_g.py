# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Two scientific-notation gaps were live: Mamba's `e`/`E` format used Rust's
# native exponent (`1.23e3`, `1.23e-4`) instead of CPython's signed,
# 2-digit-min form (`1.23e+03`, `1.23e-04`); and the `g`/`G` general type
# was missing entirely (the format-spec parser fell through to the default
# `value_to_string` path, ignoring precision and exponent cutover).
#
# Fix: post-process Rust's `{:e}` output through `pythonize_exponent` and add
# a dedicated `g`/`G` arm in `apply_format_spec` (`runtime/string_ops.rs`).
# `g` selects between `f` and `e` styles based on the decimal exponent,
# matching Python: `e` if exp < -4 or exp >= precision, else `f`. Trailing
# zeros (and a dangling `.`) are stripped unless the `#` alternate form is in
# effect. Precision 0 is treated as 1 (CPython behaviour).

# `e`/`E` — exponent is now signed and zero-padded to 2 digits min.
print(f"{1234567.89:e}")       # 1.234568e+06
print(f"{0.000123:e}")         # 1.230000e-04
print(f"{1234.5:.2e}")         # 1.23e+03
print(f"{1234.5:.2E}")         # 1.23E+03
print(f"{-1234.5:.2e}")        # -1.23e+03

# Three-digit exponents are emitted as-is (no over-padding).
print(f"{1.5e100:.2e}")        # 1.50e+100

# `g` — picks `f` style when exponent fits, `e` style otherwise.
print(f"{12345.678:g}")        # 12345.7
print(f"{0.0001:g}")           # 0.0001
print(f"{1e10:g}")             # 1e+10
print(f"{0.00001:g}")          # 1e-05
print(f"{123456.789:.3g}")     # 1.23e+05
print(f"{1.0:g}")              # 1

# `#` alternate form on `g` keeps trailing zeros and the decimal point.
print(f"{1.0:#g}")             # 1.00000
