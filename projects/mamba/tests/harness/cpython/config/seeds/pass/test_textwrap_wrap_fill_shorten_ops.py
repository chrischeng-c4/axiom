# Operational AssertionPass seed for the `textwrap` module's
# wrap / fill / shorten surface — the line-breaker / text-formatter
# functions that every CLI help-text builder, log-line formatter,
# release-note generator, and docstring-pretty-printer reaches for
# when emitting paragraphs that need to fit a fixed column width.
# Existing `test_textwrap_dedent_indent_ops.py` explicitly skips
# `wrap` / `fill` / `shorten` because mamba previously no-op'd them
# (returned the original string unchanged); the runtime has since
# implemented these and this seed pins the matching subset between
# mamba and CPython. Companion to `test_textwrap.py` (vendored
# stdlib unittest seed) and `test_textwrap_dedent_indent_ops.py`
# (existing dedent/indent ops).
#
# Surface (the matching subset between mamba and CPython):
#   • textwrap.wrap(text: str, width: int = 70) → list[str]
#       — splits `text` into a list of lines, each no longer than
#         `width`;
#       — `wrap("", w) == []` (empty input → empty list);
#       — `wrap("short", 100) == ["short"]` (already-fits → single
#         element);
#       — each element is `<= width` (when no word is longer than
#         `width`, or when `break_long_words=False` is not in play);
#       — long word with `break_long_words=False` survives as a
#         single element even when its length exceeds `width`;
#   • textwrap.fill(text: str, width: int = 70) → str
#       — equivalent to `"\n".join(wrap(text, width))`;
#       — `fill("") == ""`;
#       — `fill("short", 100) == "short"`;
#       — round-trip invariant: `fill(t, w) == "\n".join(wrap(t, w))`;
#   • textwrap.shorten(text: str, width: int) → str
#       — collapses `text` to ≤ `width` characters by replacing the
#         tail with the placeholder (`" [...]"` by default);
#       — `shorten("hi", 20) == "hi"` (already fits → unchanged);
#       — `shorten("", w) == ""`;
#       — for text that already fits (incl. exact width), returns the
#         original;
#   • return-type discipline — `wrap → list`, `fill → str`,
#     `shorten → str`;
#   • module-level attribute discipline — `wrap`, `fill`, `shorten`,
#     `dedent`, `indent` all present + callable; module name ==
#     `textwrap`.
import textwrap
_ledger: list[int] = []

# wrap — empty input returns []
assert textwrap.wrap("", 10) == []; _ledger.append(1)
assert textwrap.wrap("", 20) == []; _ledger.append(1)
assert textwrap.wrap("", 70) == []; _ledger.append(1)

# wrap — short input that fits in width
assert textwrap.wrap("abc", 10) == ["abc"]; _ledger.append(1)
assert textwrap.wrap("hello", 100) == ["hello"]; _ledger.append(1)
assert textwrap.wrap("a", 70) == ["a"]; _ledger.append(1)

# wrap — width 10 on a fixed sentence
assert textwrap.wrap("hello world this is text", 10) == ["hello", "world this", "is text"]; _ledger.append(1)

# wrap — wider widths
assert textwrap.wrap("hello world this is text", 20) == ["hello world this is", "text"]; _ledger.append(1)
assert textwrap.wrap("the quick brown fox jumps over the lazy dog", 50) == ["the quick brown fox jumps over the lazy dog"]; _ledger.append(1)

# wrap — width 5 on "the quick brown fox..."
assert textwrap.wrap("the quick brown fox jumps over the lazy dog", 5) == ["the", "quick", "brown", "fox", "jumps", "over", "the", "lazy", "dog"]; _ledger.append(1)

# wrap — width 10 on "the quick brown fox..."
assert textwrap.wrap("the quick brown fox jumps over the lazy dog", 10) == ["the quick", "brown fox", "jumps over", "the lazy", "dog"]; _ledger.append(1)

# wrap — width 15
assert textwrap.wrap("the quick brown fox jumps over the lazy dog", 15) == ["the quick brown", "fox jumps over", "the lazy dog"]; _ledger.append(1)

# wrap — width 20
assert textwrap.wrap("the quick brown fox jumps over the lazy dog", 20) == ["the quick brown fox", "jumps over the lazy", "dog"]; _ledger.append(1)

# wrap — break_long_words=False keeps long words whole
assert textwrap.wrap("abcdefghij", 5, break_long_words=False) == ["abcdefghij"]; _ledger.append(1)

# wrap — single char
assert textwrap.wrap("a", 10) == ["a"]; _ledger.append(1)
assert textwrap.wrap("x", 1) == ["x"]; _ledger.append(1)

# fill — empty input
assert textwrap.fill("", 10) == ""; _ledger.append(1)
assert textwrap.fill("", 70) == ""; _ledger.append(1)

# fill — short input that fits
assert textwrap.fill("abc", 10) == "abc"; _ledger.append(1)
assert textwrap.fill("hello", 100) == "hello"; _ledger.append(1)

# fill — width 10 on a sentence
assert textwrap.fill("hello world this is text", 10) == "hello\nworld this\nis text"; _ledger.append(1)

# fill round-trip — fill(t, w) == "\n".join(wrap(t, w))
for _t, _w in [
    ("hello world this is text", 10),
    ("the quick brown fox jumps over the lazy dog", 15),
    ("a b c d e", 5),
    ("single", 100),
]:
    assert textwrap.fill(_t, _w) == "\n".join(textwrap.wrap(_t, _w)); _ledger.append(1)

# shorten — empty input
assert textwrap.shorten("", 10) == ""; _ledger.append(1)
assert textwrap.shorten("", 70) == ""; _ledger.append(1)

# shorten — short string that already fits
assert textwrap.shorten("hi", 20) == "hi"; _ledger.append(1)
assert textwrap.shorten("abc", 100) == "abc"; _ledger.append(1)

# shorten — text already fits exactly
assert textwrap.shorten("hello world this is text", 24) == "hello world this is text"; _ledger.append(1)
assert textwrap.shorten("hello world this is text", 25) == "hello world this is text"; _ledger.append(1)
assert textwrap.shorten("hello world this is text", 100) == "hello world this is text"; _ledger.append(1)

# shorten — long text gets truncated with default placeholder " [...]"
assert textwrap.shorten("hello world this is a sentence", 20) == "hello world [...]"; _ledger.append(1)

# Return-type discipline — wrap returns list, fill/shorten return str
assert isinstance(textwrap.wrap("a", 10), list); _ledger.append(1)
assert isinstance(textwrap.wrap("hello world", 5), list); _ledger.append(1)
assert isinstance(textwrap.wrap("", 10), list); _ledger.append(1)
assert isinstance(textwrap.fill("a", 10), str); _ledger.append(1)
assert isinstance(textwrap.fill("hello world", 5), str); _ledger.append(1)
assert isinstance(textwrap.fill("", 10), str); _ledger.append(1)
assert isinstance(textwrap.shorten("a", 10), str); _ledger.append(1)
assert isinstance(textwrap.shorten("hello world this is a sentence", 20), str); _ledger.append(1)
assert isinstance(textwrap.shorten("", 10), str); _ledger.append(1)

# Length discipline — every wrap element ≤ width (when no word longer than width)
for _t, _w in [
    ("hello world this is text", 10),
    ("the quick brown fox", 8),
    ("a b c d e f g", 3),
]:
    for _line in textwrap.wrap(_t, _w):
        assert len(_line) <= _w; _ledger.append(1)

# Module-level attribute discipline
assert hasattr(textwrap, "wrap"); _ledger.append(1)
assert hasattr(textwrap, "fill"); _ledger.append(1)
assert hasattr(textwrap, "shorten"); _ledger.append(1)
assert hasattr(textwrap, "dedent"); _ledger.append(1)
assert hasattr(textwrap, "indent"); _ledger.append(1)

# Callable discipline
assert callable(textwrap.wrap); _ledger.append(1)
assert callable(textwrap.fill); _ledger.append(1)
assert callable(textwrap.shorten); _ledger.append(1)
assert callable(textwrap.dedent); _ledger.append(1)
assert callable(textwrap.indent); _ledger.append(1)

# Module name discipline
assert textwrap.__name__ == "textwrap"; _ledger.append(1)

# Idempotence — wrap output joined back into single string then re-wrapped yields same shape
_wrapped = textwrap.wrap("hello world this is text", 10)
_rewrapped = textwrap.wrap(" ".join(_wrapped), 10)
assert _wrapped == _rewrapped; _ledger.append(1)

# shorten — wider widths yield more content
_s_long = "hello world this is a long sentence with many words"
_short_10 = textwrap.shorten(_s_long, 10)
_short_30 = textwrap.shorten(_s_long, 30)
_short_100 = textwrap.shorten(_s_long, 100)
assert len(_short_10) <= len(_short_30) <= len(_short_100); _ledger.append(1)

# shorten returns original text when width is wide enough
assert textwrap.shorten(_s_long, len(_s_long) + 100) == _s_long; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_textwrap_wrap_fill_shorten_ops {sum(_ledger)} asserts")
