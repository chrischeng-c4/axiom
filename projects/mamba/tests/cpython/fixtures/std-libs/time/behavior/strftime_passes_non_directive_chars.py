# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "strftime_passes_non_directive_chars"
# subject = "time.strftime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time.strftime: strftime copies non-directive characters verbatim — bare emoji, lone/paired UTF-16 surrogates, and embedded NUL bytes (single, 1000-run, and interleaved with %c/%B directives) pass through unchanged; only %-directives are interpreted"""
import time

tt = time.gmtime(0)
# Resolve the two locale-dependent directives once, then assert that
# surrounding literal text is preserved around them.
c = time.strftime("%c", tt)
b = time.strftime("%B", tt)

# A bare emoji is not a directive -> passes through unchanged.
assert time.strftime("\U0001f40d", tt) == "\U0001f40d", "emoji passthrough"

# Emoji interleaved with directives keeps both literal and formatted parts.
assert time.strftime("\U0001f4bb%c\U0001f40d%B", tt) == f"\U0001f4bb{c}\U0001f40d{b}", \
    "emoji + directives"
assert time.strftime("%c\U0001f4bb%B\U0001f40d", tt) == f"{c}\U0001f4bb{b}\U0001f40d", \
    "directives + emoji"

# Lone surrogate code points survive intact (str can hold them).
assert time.strftime("\ud83d", tt) == "\ud83d", "lone high surrogate"
assert time.strftime("\udc0d", tt) == "\udc0d", "lone low surrogate"
assert time.strftime("\U0001f40d", tt) == "\U0001f40d", "surrogate pair literal"
assert time.strftime("\ud83d%c\udc0d%B", tt) == f"\ud83d{c}\udc0d{b}", \
    "surrogates + directives"

# Embedded NUL is preserved, including long runs and around directives.
assert time.strftime("\x00", tt) == "\x00", "single NUL"
assert time.strftime("\x00" * 1000, tt) == "\x00" * 1000, "NUL run"
assert time.strftime("\x00%c\x00%B", tt) == f"\x00{c}\x00{b}", "NUL + directives"
print("strftime_passes_non_directive_chars OK")
