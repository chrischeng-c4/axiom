# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Two format-spec gaps were live: f-strings/format() ignored the `#` alternate
# form (`f"{255:#x}"` returned `"255"` instead of `"0xff"`), and the `%`
# percent type was completely unimplemented (`"{:.0%}".format(0.5)` → `"0.5"`).
#
# Root cause: `runtime/string_ops.rs` had two parallel format-spec parsers —
# `format_with_spec` (for f-strings + `format()`) was missing # / sign / _ /
# %; `apply_format_spec` (for `"{}".format(...)`) had # but lacked %.
# Fix unifies both behind `apply_format_spec` and adds a `%` arm.

# `#` alternate form on f-strings — was returning the raw int.
print(f"{255:#x}")            # 0xff
print(f"{255:#X}")            # 0XFF
print(f"{8:#o}")              # 0o10
print(f"{5:#b}")              # 0b101

# Same on the `format()` builtin (was: same broken path as f-strings).
print(format(255, "#x"))      # 0xff
print(format(8, "#o"))         # 0o10

# `%` percent type — multiply by 100, format as fixed-point, append "%".
print(f"{0.5:.0%}")            # 50%
print(f"{0.123:.1%}")          # 12.3%
print(f"{1.0:.2%}")            # 100.00%
print("{:.0%}".format(0.5))   # 50%

# `_` thousands separator now flows through both paths.
print(f"{1234567:_}")         # 1_234_567

# Pre-existing `,` separator path still works.
print(f"{1234567:,}")         # 1,234,567
