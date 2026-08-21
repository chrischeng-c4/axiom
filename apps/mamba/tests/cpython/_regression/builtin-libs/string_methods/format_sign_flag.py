# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Format-spec sign flag (`+`, ` `) when no explicit type char is given.
#
# Bug: `apply_format_spec` had a unified `'s' | '\0'` arm that, when
# `thousands` was None, fell through to a string-style path that built
# the body via `value_to_string(val)` and emitted an empty `sign_prefix`.
# That meant `f"{42:+}"` printed `42` (no `+`) and `f"{42: }"` printed
# `42` (no leading space). CPython treats `+`/` ` as numeric sign flags
# and produces `"+42"` / `" 42"` even with no `d`/`f` suffix.
#
# Fix: when sign is `+` or space and the value is int/float, format like
# `d` / `f` would (sign prefix + abs body). The default `-` path keeps
# the existing string fallback so non-numerics aren't perturbed.

# Plain integers.
print(f"{42:+}")           # +42
print(f"{-42:+}")          # -42
print(f"{0:+}")            # +0
print(f"{42: }")           #  42  (leading space)
print(f"{-42: }")          # -42
print(f"{0: }")            #  0

# Plain floats.
print(f"{3.14:+}")         # +3.14
print(f"{-3.14:+}")        # -3.14
print(f"{0.0:+}")          # +0.0
print(f"{3.14: }")         #  3.14
print(f"{-3.14: }")        # -3.14

# Width + sign + zero-pad combine correctly.
print(f"{42:+05}")         # +0042
print(f"{-42:+05}")        # -0042
print(f"{42: 05}")         #  0042

# Width + sign without zero-fill — right-aligned by default for numerics.
print(f"{42:+6}")          #     +42
print(f"{-42:+6}")          #     -42

# Default (no sign flag) is unchanged.
print(f"{42}")             # 42
print(f"{-42}")            # -42

# Strings ignore the numeric-sign branch — the `s` fallback still applies.
# (Python actually rejects `+` on strings with ValueError, but we accept
# it silently and just drop it; the important contract is that we don't
# crash and the body is preserved.)
print(f"{'hi'}")           # hi
