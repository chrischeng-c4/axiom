# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Regression: str.translate and str.format_map were missing from the
# method dispatcher — translate returned None, format_map raised
# AttributeError.

# str.maketrans → translate
tbl = str.maketrans("abc", "xyz")
print("cat bat".translate(tbl))
print("banana".translate(str.maketrans("an", "AN")))

# translate with deletion (third arg to maketrans)
# str.maketrans(x, y, z) — remove chars in z
tbl2 = str.maketrans("", "", "aeiou")
print("hello world".translate(tbl2))

# format_map
print("{name} is {age}".format_map({"name": "Alice", "age": 30}))

# format_map with missing key raises KeyError
try:
    "{missing}".format_map({})
except KeyError as e:
    print("caught:", e)
