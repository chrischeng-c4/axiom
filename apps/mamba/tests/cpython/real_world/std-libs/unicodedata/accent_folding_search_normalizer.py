# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "real_world"
# case = "accent_folding_search_normalizer"
# subject = "unicodedata"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata: a diacritic-insensitive search helper folds accents by NFD-decomposing text and dropping combining marks (category 'Mn'), so 'cafe' matches an accented 'cafe' corpus entry"""
import unicodedata

def fold(text):
    """Diacritic-insensitive fold: NFD then drop combining marks (Mn)."""
    decomposed = unicodedata.normalize("NFD", text)
    return "".join(ch for ch in decomposed
                   if unicodedata.category(ch) != "Mn").casefold()


corpus = ["Café Über", "naïve", "RÉSUMÉ", "jalapeño", "plain text"]
folded = [fold(entry) for entry in corpus]

# Accented queries match their accented corpus entries after folding.
assert fold("cafe uber") == folded[0], f"cafe uber -> {folded[0]!r}"
assert fold("NAIVE") == folded[1], f"naive -> {folded[1]!r}"
assert fold("resume") == folded[2], f"resume -> {folded[2]!r}"
assert fold("JALAPENO") == folded[3], f"jalapeno -> {folded[3]!r}"

# A search over the folded index finds accented entries from ASCII queries.
def search(query):
    target = fold(query)
    return [corpus[i] for i, f in enumerate(folded) if target in f]

assert search("cafe") == ["Café Über"], f"search cafe -> {search('cafe')!r}"
assert search("naive") == ["naïve"], f"search naive -> {search('naive')!r}"
assert search("resume") == ["RÉSUMÉ"], f"search resume -> {search('resume')!r}"
assert search("xyz") == [], "no spurious matches"

print("accent_folding_search_normalizer OK")
