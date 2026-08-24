use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/unicodedata/category_letter_case_classes.py`.
#[test]
fn test_gen_behavior_std_libs_unicodedata_category_letter_case_classes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "category_letter_case_classes"
# subject = "unicodedata.category"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.category: category returns Lu/Ll/Lt for uppercase 'A', lowercase 'a', and titlecase digraph (U+01F2)"""
import unicodedata

assert unicodedata.category("A") == "Lu", "uppercase"
assert unicodedata.category("a") == "Ll", "lowercase"
assert unicodedata.category("ǲ") == "Lt", "titlecase Dz"  # U+01F2

print("category_letter_case_classes OK")
"###);
    assert_output(&out, r###"category_letter_case_classes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unicodedata/combining_class_zero_for_base_nonzero_for_marks.py`.
#[test]
fn test_gen_behavior_std_libs_unicodedata_combining_class_zero_for_base_nonzero_for_marks() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "combining_class_zero_for_base_nonzero_for_marks"
# subject = "unicodedata.combining"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.combining: combining is 0 for base letters A/Z and positive for the combining grave (U+0300) and acute (U+0301) marks"""
import unicodedata

assert unicodedata.combining("A") == 0, "base letter A combining = 0"
assert unicodedata.combining("Z") == 0, "base letter Z combining = 0"
assert unicodedata.combining("̀") > 0, "combining grave > 0"
assert unicodedata.combining("́") > 0, "combining acute > 0"

print("combining_class_zero_for_base_nonzero_for_marks OK")
"###);
    assert_output(&out, r###"combining_class_zero_for_base_nonzero_for_marks OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unicodedata/decomposition_fraction_tag_format.py`.
#[test]
fn test_gen_behavior_std_libs_unicodedata_decomposition_fraction_tag_format() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "decomposition_fraction_tag_format"
# subject = "unicodedata.decomposition"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.decomposition: the vulgar fraction one quarter (U+00BC) decomposes to the tagged form '<fraction> 0031 2044 0034'"""
import unicodedata

assert unicodedata.decomposition("¼") == "<fraction> 0031 2044 0034", (
    f"one-quarter decomposition = {unicodedata.decomposition(chr(0x00bc))!r}"
)

print("decomposition_fraction_tag_format OK")
"###);
    assert_output(&out, r###"decomposition_fraction_tag_format OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unicodedata/decomposition_string_for_precomposed.py`.
#[test]
fn test_gen_behavior_std_libs_unicodedata_decomposition_string_for_precomposed() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "decomposition_string_for_precomposed"
# subject = "unicodedata.decomposition"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.decomposition: decomposition('A') is empty while e-acute yields a non-empty mapping starting with code point 0065"""
import unicodedata

assert unicodedata.decomposition("A") == "", f"A decomposition = {unicodedata.decomposition('A')!r}"
_d = unicodedata.decomposition("é")  # precomposed e-acute
assert _d != "", f"e-acute has decomposition = {_d!r}"
assert _d.startswith("0065"), f"decomposition starts with base 'e' = {_d!r}"

print("decomposition_string_for_precomposed OK")
"###);
    assert_output(&out, r###"decomposition_string_for_precomposed OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unicodedata/east_asian_width_width_classes.py`.
#[test]
fn test_gen_behavior_std_libs_unicodedata_east_asian_width_width_classes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "east_asian_width_width_classes"
# subject = "unicodedata.east_asian_width"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.east_asian_width: east_asian_width returns Na for space, W for a CJK char (U+C894), H for halfwidth (U+FF66), F for fullwidth (U+FF1F)"""
import unicodedata

assert unicodedata.east_asian_width("\x20") == "Na", "space is narrow"
assert unicodedata.east_asian_width("좔") == "W", "CJK is wide"
assert unicodedata.east_asian_width("ｦ") == "H", "halfwidth katakana is H"
assert unicodedata.east_asian_width("？") == "F", "fullwidth question mark is F"
assert unicodedata.east_asian_width("‐") == "A", "hyphen is ambiguous"

print("east_asian_width_width_classes OK")
"###);
    assert_output(&out, r###"east_asian_width_width_classes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unicodedata/hangul_algorithmic_round_trip.py`.
#[test]
fn test_gen_behavior_std_libs_unicodedata_hangul_algorithmic_round_trip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "hangul_algorithmic_round_trip"
# subject = "unicodedata.normalize"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.normalize: precomposed Hangul syllables decompose to six jamo under NFD and recompose losslessly under NFC (no UCD table entries)"""
import unicodedata

_hangul = "한글"  # two precomposed Hangul syllables
_hangul_nfd = unicodedata.normalize("NFD", _hangul)
assert len(_hangul_nfd) == 6, f"Hangul NFD len = {len(_hangul_nfd)!r}"
assert unicodedata.normalize("NFC", _hangul_nfd) == _hangul, "Hangul NFC round-trip"
assert unicodedata.normalize("NFC", _hangul) == _hangul, "already-composed Hangul stable"

print("hangul_algorithmic_round_trip OK")
"###);
    assert_output(&out, r###"hangul_algorithmic_round_trip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unicodedata/is_normalized_nfc_nfd_distinguishes_forms.py`.
#[test]
fn test_gen_behavior_std_libs_unicodedata_is_normalized_nfc_nfd_distinguishes_forms() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "is_normalized_nfc_nfd_distinguishes_forms"
# subject = "unicodedata.is_normalized"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.is_normalized: is_normalized reports True/False correctly for precomposed vs decomposed e-acute under NFC and NFD"""
import unicodedata

_precomp = "é"     # precomposed e-acute (U+00E9)
_decomp = "é"     # decomposed: 'e' + combining acute (U+0301)
assert unicodedata.is_normalized("NFC", _precomp) is True, "precomposed is NFC"
assert unicodedata.is_normalized("NFD", _precomp) is False, "precomposed not NFD"
assert unicodedata.is_normalized("NFD", _decomp) is True, "decomposed is NFD"
assert unicodedata.is_normalized("NFC", _decomp) is False, "decomposed not NFC"

print("is_normalized_nfc_nfd_distinguishes_forms OK")
"###);
    assert_output(&out, r###"is_normalized_nfc_nfd_distinguishes_forms OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unicodedata/issue29456_hangul_recompose.py`.
#[test]
fn test_gen_behavior_std_libs_unicodedata_issue29456_hangul_recompose() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "issue29456_hangul_recompose"
# subject = "unicodedata.normalize"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.normalize: NFC of Hangul jamo sequences recomposes per issue #29456 (e.g. choseong+jungseong U+1100 U+1175 + jongseong collapses to the precomposed syllable)"""
import unicodedata

# issue #29456: Hangul jamo NFC recomposition corner cases.
# An L+V+T whose T composes stays as one syllable.
assert (unicodedata.normalize("NFC", "ᄀᅶᆨ")
        == "ᄀᅶᆨ"), "u1176 sequence stable"
# An L+V recomposes to a syllable; a trailing non-composing jongseong stays.
assert (unicodedata.normalize("NFC", "기ᆧ")
        == "기ᆧ"), "u11a7 L+V recomposes, T trails"
assert (unicodedata.normalize("NFC", "기ᇃ")
        == "기ᇃ"), "u11c3 L+V recomposes, T trails"

print("issue29456_hangul_recompose OK")
"###);
    assert_output(&out, r###"issue29456_hangul_recompose OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unicodedata/mirrored_flags_bracket_chars.py`.
#[test]
fn test_gen_behavior_std_libs_unicodedata_mirrored_flags_bracket_chars() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "mirrored_flags_bracket_chars"
# subject = "unicodedata.mirrored"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.mirrored: mirrored is 0 for 'A' and 1 for the open parenthesis '(' (a bidi-mirrored bracket)"""
import unicodedata

assert unicodedata.mirrored("A") == 0, f"A not mirrored = {unicodedata.mirrored('A')!r}"
assert unicodedata.mirrored("(") == 1, f"( is mirrored = {unicodedata.mirrored('(')!r}"

print("mirrored_flags_bracket_chars OK")
"###);
    assert_output(&out, r###"mirrored_flags_bracket_chars OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unicodedata/name_default_for_unnamed_char.py`.
#[test]
fn test_gen_behavior_std_libs_unicodedata_name_default_for_unnamed_char() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "name_default_for_unnamed_char"
# subject = "unicodedata.name"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.name: name() returns the supplied default for an unnamed control character (NUL) instead of raising"""
import unicodedata

_n = unicodedata.name(chr(0), "NULL")  # NUL has no Unicode name
assert _n == "NULL", f"name NUL default = {_n!r}"

print("name_default_for_unnamed_char OK")
"###);
    assert_output(&out, r###"name_default_for_unnamed_char OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unicodedata/name_lookup_round_trip.py`.
#[test]
fn test_gen_behavior_std_libs_unicodedata_name_lookup_round_trip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "name_lookup_round_trip"
# subject = "unicodedata.lookup"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.lookup: name() and lookup() are inverses over a sample of named characters (A Z 0 9 e-acute n-tilde alpha)"""
import unicodedata

for _ch in ["A", "Z", "0", "9", "é", "ñ", "α"]:
    _nm = unicodedata.name(_ch, None)
    assert _nm is not None, f"sample char {_ch!r} should be named"
    assert unicodedata.lookup(_nm) == _ch, f"round-trip {_ch!r} via {_nm!r}"

print("name_lookup_round_trip OK")
"###);
    assert_output(&out, r###"name_lookup_round_trip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unicodedata/nfc_composes_to_single_char.py`.
#[test]
fn test_gen_behavior_std_libs_unicodedata_nfc_composes_to_single_char() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "nfc_composes_to_single_char"
# subject = "unicodedata.normalize"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.normalize: NFC recomposes e-acute back into the single precomposed code point (length 1)"""
import unicodedata

_decomposed = "é"  # 'e' + combining acute
_precomposed = unicodedata.normalize("NFC", _decomposed)
assert _precomposed == "é", f"NFC compose = {_precomposed!r}"
assert len(_precomposed) == 1, f"NFC len = {len(_precomposed)!r}"

print("nfc_composes_to_single_char OK")
"###);
    assert_output(&out, r###"nfc_composes_to_single_char OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unicodedata/nfd_decomposes_to_base_plus_mark.py`.
#[test]
fn test_gen_behavior_std_libs_unicodedata_nfd_decomposes_to_base_plus_mark() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "nfd_decomposes_to_base_plus_mark"
# subject = "unicodedata.normalize"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.normalize: NFD splits precomposed e-acute into base 'e' (U+0065) followed by combining acute (U+0301)"""
import unicodedata

_nfd = unicodedata.normalize("NFD", "é")  # precomposed e-acute
assert len(_nfd) == 2, f"NFD length = {len(_nfd)!r}"
assert ord(_nfd[0]) == 0x65, f"NFD base = {ord(_nfd[0]):#x}"
assert ord(_nfd[1]) == 0x301, f"NFD mark = {ord(_nfd[1]):#x}"

print("nfd_decomposes_to_base_plus_mark OK")
"###);
    assert_output(&out, r###"nfd_decomposes_to_base_plus_mark OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unicodedata/nfkd_expands_compatibility_ligature.py`.
#[test]
fn test_gen_behavior_std_libs_unicodedata_nfkd_expands_compatibility_ligature() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "nfkd_expands_compatibility_ligature"
# subject = "unicodedata.normalize"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.normalize: NFKD maps the fi ligature (U+FB01) to the ASCII pair 'fi'"""
import unicodedata

_nfkd = unicodedata.normalize("NFKD", "ﬁ")  # fi ligature U+FB01
assert _nfkd == "fi", f"NFKD ligature = {_nfkd!r}"

print("nfkd_expands_compatibility_ligature OK")
"###);
    assert_output(&out, r###"nfkd_expands_compatibility_ligature OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unicodedata/numeric_digit_decimal_values.py`.
#[test]
fn test_gen_behavior_std_libs_unicodedata_numeric_digit_decimal_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "numeric_digit_decimal_values"
# subject = "unicodedata.numeric"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.numeric: digit('5')==5, decimal('7')==7, numeric vulgar-half (U+00BD)==0.5"""
import unicodedata

assert unicodedata.digit("5") == 5, f"digit '5' = {unicodedata.digit('5')!r}"
assert unicodedata.decimal("7") == 7, f"decimal '7' = {unicodedata.decimal('7')!r}"
assert unicodedata.numeric("½") == 0.5, f"numeric half = {unicodedata.numeric('½')!r}"

print("numeric_digit_decimal_values OK")
"###);
    assert_output(&out, r###"numeric_digit_decimal_values OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unicodedata/pr29_composition_is_stable.py`.
#[test]
fn test_gen_behavior_std_libs_unicodedata_pr29_composition_is_stable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "pr29_composition_is_stable"
# subject = "unicodedata.normalize"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.normalize: PR-29 sequences are stable under NFC: normalize('NFC', text) returns text for the documented composition-exclusion cases"""
import unicodedata

# https://www.unicode.org/review/pr-29.html (issues #1054943, #10254):
# these sequences must be NFC-stable.
composed = (
    "େ̀ା",
    "ᄀ̀ᅡ",
    "Li̍t-sṳ́",
    "मार्क ज़"
    + "ुकेरबर्ग",
    "किर्गिज़"
    + "स्तान",
)
for text in composed:
    assert unicodedata.normalize("NFC", text) == text, f"PR-29 stable: {text!r}"

# issue #10254: a long run of C+combining-marks must not crash and is stable.
a = "C̸" * 20 + "Ç"
b = "C̸" * 20 + "\xC7"
assert unicodedata.normalize("NFC", a) == b, "issue10254 NFC"

print("pr29_composition_is_stable OK")
"###);
    assert_output(&out, r###"pr29_composition_is_stable OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unicodedata/splitlines_unicode_line_boundaries.py`.
#[test]
fn test_gen_behavior_std_libs_unicodedata_splitlines_unicode_line_boundaries() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "splitlines_unicode_line_boundaries"
# subject = "str.splitlines"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""str.splitlines: str.splitlines breaks on exactly the Unicode line-boundary set (LF/VT/FF/CR/FS/GS/RS/NEL/LS/PS) over the BMP, not just ASCII newlines"""
import unicodedata

# bug 7643: the code points str.splitlines treats as line boundaries.
BREAKERS = {
    0x0A,   # LINE FEED
    0x0B,   # LINE TABULATION
    0x0C,   # FORM FEED
    0x0D,   # CARRIAGE RETURN
    0x1C,   # FILE SEPARATOR
    0x1D,   # GROUP SEPARATOR
    0x1E,   # RECORD SEPARATOR
    0x85,   # NEXT LINE
    0x2028,  # LINE SEPARATOR
    0x2029,  # PARAGRAPH SEPARATOR
}
# A code point is a line boundary iff "<ch>A" splits into two pieces.
found = {i for i in range(0x10000) if len((chr(i) + "A").splitlines()) == 2}
assert found == BREAKERS, f"line-boundary set mismatch: {sorted(found ^ BREAKERS)!r}"

# Spot-check breakers and non-breakers explicitly.
assert "a\nb".splitlines() == ["a", "b"], "LF splits"
assert "a b".splitlines() == ["a", "b"], "LINE SEPARATOR splits"
assert "a b".splitlines() == ["a b"], "plain space does not split"
assert "a\tb".splitlines() == ["a\tb"], "tab does not split"

print("splitlines_unicode_line_boundaries OK")
"###);
    assert_output(&out, r###"splitlines_unicode_line_boundaries OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unicodedata/ucd_3_2_0_pins_old_unicode_version.py`.
#[test]
fn test_gen_behavior_std_libs_unicodedata_ucd_3_2_0_pins_old_unicode_version() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "ucd_3_2_0_pins_old_unicode_version"
# subject = "unicodedata.ucd_3_2_0"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.ucd_3_2_0: the frozen ucd_3_2_0 database reports unidata_version '3.2.0' and disagrees with the live UCD on a post-3.2 mirrored change (U+0F3A) while stable properties agree"""
import unicodedata

ucd_old = unicodedata.ucd_3_2_0

# The pinned database carries its own (older) version string.
assert ucd_old.unidata_version == "3.2.0", f"pinned version = {ucd_old.unidata_version!r}"
assert unicodedata.unidata_version != "3.2.0", "live UCD is newer than 3.2.0"

# bug ucd_510: U+0F3A became mirrored after Unicode 3.2, so the two
# databases disagree on .mirrored().
_ch = "༺"  # TIBETAN MARK GTER YIG MGO UM RNAM BCAD MA
assert unicodedata.mirrored(_ch) == 1, f"live mirrored = {unicodedata.mirrored(_ch)!r}"
assert ucd_old.mirrored(_ch) == 0, f"3.2.0 mirrored = {ucd_old.mirrored(_ch)!r}"

# Properties that predate Unicode 3.2 match in both databases.
for _q in (lambda u: u.name("A"),
           lambda u: u.category("A"),
           lambda u: u.bidirectional("A"),
           lambda u: u.combining("A")):
    assert _q(unicodedata) == _q(ucd_old), "stable property agrees across UCD versions"

print("ucd_3_2_0_pins_old_unicode_version OK")
"###);
    assert_output(&out, r###"ucd_3_2_0_pins_old_unicode_version OK
"###);
}
