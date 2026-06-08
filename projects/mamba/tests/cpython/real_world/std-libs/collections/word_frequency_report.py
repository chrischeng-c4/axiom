# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "real_world"
# case = "word_frequency_report"
# subject = "collections.Counter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.Counter: a log/text analytics job tokenizes a paragraph, builds a Counter word-frequency table, ranks the top tokens with most_common, groups lines by length with defaultdict(list), and rolls a fixed-size recent-window with a maxlen deque"""
from collections import Counter, defaultdict, deque

text = (
    "the quick brown fox the lazy dog the quick fox\n"
    "a quick brown dog jumps over the lazy fox\n"
    "the dog and the fox are quick friends"
)

# 1. Tokenize and tally word frequencies.
words = text.split()
freq = Counter(words)
assert freq["the"] == 6, f"the = {freq['the']!r}"
assert freq["quick"] == 4, f"quick = {freq['quick']!r}"
assert freq["fox"] == 4, f"fox = {freq['fox']!r}"

# 2. Rank the most common tokens.
top = freq.most_common(3)
assert top[0] == ("the", 6), f"top token = {top[0]!r}"
assert {w for w, _ in top} == {"the", "quick", "fox"}, f"top three = {top!r}"

# 3. Group lines by their word count with a defaultdict(list).
by_len = defaultdict(list)
for line in text.splitlines():
    by_len[len(line.split())].append(line)
assert sorted(by_len) == [8, 9, 10], f"line lengths = {sorted(by_len)!r}"
assert len(by_len[9]) == 1 and by_len[9][0].startswith("a quick"), "one nine-word line"

# 4. Roll a fixed-size recent-window over the token stream.
window = deque(maxlen=3)
for w in words:
    window.append(w)
assert len(window) == 3, "window bounded at maxlen"
assert list(window) == words[-3:], f"window = {list(window)!r}"

print("word_frequency_report OK")
