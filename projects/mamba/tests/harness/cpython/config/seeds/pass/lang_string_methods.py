# Operational AssertionPass seed for str method surfaces.
# Surface: case mutation (upper/lower/swapcase/title/capitalize/case-
# fold); whitespace and char-set stripping (strip/lstrip/rstrip/strip
# chars); replace (with and without count limit); split (default
# whitespace, explicit separator, maxsplit), rsplit, splitlines;
# join, startswith / endswith (single needle and tuple-of-needles);
# find / index / rfind / count; classification predicates (isdigit /
# isalpha / isalnum / isspace / isupper / islower); padding (zfill /
# center / ljust / rjust); encode → bytes; format with positional
# {} and named {n}; PEP 616 removeprefix / removesuffix; expandtabs;
# str.maketrans + translate; partition / rpartition.
_ledger: list[int] = []

# Case mutation
assert "hello".upper() == "HELLO"; _ledger.append(1)
assert "HELLO".lower() == "hello"; _ledger.append(1)
assert "Hello".swapcase() == "hELLO"; _ledger.append(1)
assert "hello world".title() == "Hello World"; _ledger.append(1)
assert "hello world".capitalize() == "Hello world"; _ledger.append(1)
assert "HeLLo".casefold() == "hello"; _ledger.append(1)

# Stripping
assert "  hello  ".strip() == "hello"; _ledger.append(1)
assert "  hello  ".lstrip() == "hello  "; _ledger.append(1)
assert "  hello  ".rstrip() == "  hello"; _ledger.append(1)
assert "xxhelloxx".strip("x") == "hello"; _ledger.append(1)

# Replace — default unlimited, then count-limited
assert "hello world".replace("world", "there") == "hello there"; _ledger.append(1)
assert "aaaa".replace("a", "b", 2) == "bbaa"; _ledger.append(1)

# Split — default whitespace, explicit separator, maxsplit
assert "a,b,c".split(",") == ["a", "b", "c"]; _ledger.append(1)
assert "a b c".split() == ["a", "b", "c"]; _ledger.append(1)
assert "a,b,c,d".split(",", 2) == ["a", "b", "c,d"]; _ledger.append(1)
assert "a,b,c,d".rsplit(",", 1) == ["a,b,c", "d"]; _ledger.append(1)

# splitlines on a multi-line string
assert "a\nb\nc".splitlines() == ["a", "b", "c"]; _ledger.append(1)

# Join — separator-aware concatenation; empty separator concatenates
assert ",".join(["a", "b", "c"]) == "a,b,c"; _ledger.append(1)
assert "".join(["a", "b"]) == "ab"; _ledger.append(1)
assert "-".join([]) == ""; _ledger.append(1)

# startswith / endswith — single needle and tuple-of-needles
assert "hello".startswith("he") == True; _ledger.append(1)
assert "hello".endswith("lo") == True; _ledger.append(1)
assert "hello".startswith(("he", "wo")) == True; _ledger.append(1)
assert "hello".endswith("hi") == False; _ledger.append(1)

# find / index / rfind / count
assert "hello".find("l") == 2; _ledger.append(1)
assert "hello".find("z") == -1; _ledger.append(1)
assert "hello".rfind("l") == 3; _ledger.append(1)
assert "hello".index("l") == 2; _ledger.append(1)
assert "abcabc".count("a") == 2; _ledger.append(1)
assert "hello".count("z") == 0; _ledger.append(1)

# Classification predicates
assert "123".isdigit() == True; _ledger.append(1)
assert "abc".isalpha() == True; _ledger.append(1)
assert "abc123".isalnum() == True; _ledger.append(1)
assert "   ".isspace() == True; _ledger.append(1)
assert "ABC".isupper() == True; _ledger.append(1)
assert "abc".islower() == True; _ledger.append(1)

# Padding — zfill, center, ljust, rjust
assert "42".zfill(5) == "00042"; _ledger.append(1)
assert "ab".center(6, "-") == "--ab--"; _ledger.append(1)
assert "ab".ljust(5, ".") == "ab..."; _ledger.append(1)
assert "ab".rjust(5, ".") == "...ab"; _ledger.append(1)

# encode → bytes
assert "hello".encode("utf-8") == b"hello"; _ledger.append(1)

# format with positional and named placeholders
assert "{} {}".format("a", "b") == "a b"; _ledger.append(1)
assert "{n}".format(n=42) == "42"; _ledger.append(1)

# PEP 616 — removeprefix / removesuffix (Python 3.9+)
assert "hello world".removeprefix("hello ") == "world"; _ledger.append(1)
assert "hello world".removesuffix(" world") == "hello"; _ledger.append(1)
# If the prefix/suffix doesn't match, the string is returned unchanged
assert "hello".removeprefix("xy") == "hello"; _ledger.append(1)
assert "hello".removesuffix("xy") == "hello"; _ledger.append(1)

# expandtabs — replace \t with the appropriate number of spaces
assert "a\tb".expandtabs(4) == "a   b"; _ledger.append(1)

# str.maketrans + translate — char-for-char translation table
tr = str.maketrans("abc", "xyz")
assert "abc".translate(tr) == "xyz"; _ledger.append(1)
assert "abc abc".translate(tr) == "xyz xyz"; _ledger.append(1)

# partition / rpartition — split at first / last occurrence into a tuple
assert "a-b-c".partition("-") == ("a", "-", "b-c"); _ledger.append(1)
assert "a-b-c".rpartition("-") == ("a-b", "-", "c"); _ledger.append(1)
# If the separator isn't found, partition returns (whole, "", "")
assert "abc".partition("-") == ("abc", "", ""); _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_string_methods {sum(_ledger)} asserts")
