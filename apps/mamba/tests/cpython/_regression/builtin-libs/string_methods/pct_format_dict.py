# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# `"%(key)s" % {"key": value}` — printf-style formatting with a mapping
# argument was unimplemented. The parser flattened the dict into a single
# arg slot (because the dict path didn't match the Tuple branch), then
# emitted the literal `%(name)s` because the parser had no `(name)` handler.
#
# Fix in `runtime/string_ops.rs::mb_str_percent_format`:
# 1. When `args` is a Dict, route it into a `mapping: Option<MbValue>` lane
#    instead of `arg_slots`.
# 2. After consuming the leading `%`, if the next char is `(`, parse the
#    parenthesised key (with depth tracking for embedded parens) and look
#    it up in the mapping. The remaining flag/width/precision/conversion
#    pipeline is reused unchanged — the only change is which value feeds
#    into the conversion.
# 3. The `arg_idx` counter advances only for tuple-style conversions, not
#    mapping conversions.

# Single-key substitution.
print("%(name)s" % {"name": "Alice"})

# Multiple keys, mixed conversions.
print("Hello, %(name)s! You are %(age)d." %
      {"name": "Bob", "age": 30})

# Width / precision applied to mapping values.
print("%(x)05d / %(y).2f" % {"x": 7, "y": 3.14159})

# Same key referenced twice — `arg_idx` does not advance for mapping
# conversions, so this works correctly.
print("%(repeat)s%(repeat)s" % {"repeat": "ab"})

# Numeric conversions on mapping values.
print("%(n)#x" % {"n": 255})
print("%(p)+d" % {"p": 42})

# Tuple-style still works (no regression).
print("%s and %s" % ("a", "b"))
print("%d / %.2f" % (5, 1.5))

# %% literal still works inside a mapping template.
print("100%% of %(t)s" % {"t": "users"})
