# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# `repr(str)` was emitting raw bytes for control characters above 0x1f.
# CPython escapes ASCII DEL (0x7f) and the C1 control range (0x80-0x9f) as
# `\xNN`, just like the C0 controls — but Mamba's repr only escaped chars
# `< 0x20`, so `repr("\x7f")` came back as `"''"` (the DEL byte verbatim
# between the quotes) and `repr("\x80")` mojibake'd similarly.
#
# Fix in `runtime/builtins.rs::mb_repr` and `runtime/string_ops.rs`'s
# `repr_in_container` (the in-list/dict path): replace the `< 0x20` check
# with `c.is_control()`, which covers C0 + C1 + (for completeness) any
# higher-plane control codepoints, escaping >= 0x100 as `\uNNNN`.

# C0 controls (already worked) — make sure we didn't regress.
print(repr("\x01\x02"))                         # '\x01\x02'
print(repr("\x1f"))                             # '\x1f'

# DEL — was the headline bug; printed empty quotes before.
print(repr("\x7f"))                             # '\x7f'

# C1 controls — also unescaped before.
print(repr("\x80"))                             # '\x80'
print(repr("\x9f"))                             # '\x9f'

# Mixed printable + control.
print(repr("hello\x7fworld"))                   # 'hello\x7fworld'

# Printable Unicode kept verbatim (Python 3 keeps non-ASCII letters as-is).
print(repr("é"))                                # 'é'
print(repr("中"))                               # '中'

# Already-handled escapes — keep their named forms, not raw \xNN.
print(repr("\n"))                               # '\n'
print(repr("\t"))                               # '\t'
print(repr("\r"))                               # '\r'
