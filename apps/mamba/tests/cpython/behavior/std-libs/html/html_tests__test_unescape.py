# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html"
# dimension = "behavior"
# case = "html_tests__test_unescape"
# subject = "cpython.test_html.HtmlTests.test_unescape"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_html.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_html.py::HtmlTests::test_unescape
"""Auto-ported test: HtmlTests::test_unescape (CPython 3.12 oracle)."""


import html


numeric_formats = ["&#%d", "&#%d;", "&#x%x", "&#x%x;"]


def check(text, expected):
    actual = html.unescape(text)
    assert actual == expected, f"unescape({text!r}) returned {actual!r}, expected {expected!r}"


def check_num(num, expected):
    for fmt in numeric_formats:
        text = fmt % num
        check(text, expected)


check("no character references", "no character references")
check("&\n&\t& &&", "&\n&\t& &&")
check("&0 &9 &a &0; &9; &a;", "&0 &9 &a &0; &9; &a;")

for value in ["&", "&#", "&#x", "&#X", "&#y", "&#xy", "&#Xy"]:
    check(value, value)
    check(value + ";", value + ";")

formats = [
    "&#%d",
    "&#%07d",
    "&#%d;",
    "&#%07d;",
    "&#x%x",
    "&#x%06x",
    "&#x%x;",
    "&#x%06x;",
    "&#x%X",
    "&#x%06X",
    "&#X%x;",
    "&#X%06x;",
]
for num, char in zip(
    [65, 97, 34, 38, 0x2603, 0x101234],
    ["A", "a", '"', "&", "\u2603", "\U00101234"],
):
    for fmt in formats:
        check(fmt % num, char)
        for end in [" ", "X"]:
            check((fmt + end) % num, char + end)

for cp in [0xD800, 0xDB00, 0xDC00, 0xDFFF, 0x110000]:
    check_num(cp, "\uFFFD")

for cp in [0x1, 0xB, 0xE, 0x7F, 0xFFFE, 0xFFFF, 0x10FFFE, 0x10FFFF]:
    check_num(cp, "")

for num, char in zip([0x0D, 0x80, 0x95, 0x9D], "\r\u20ac\u2022\x9d"):
    check_num(num, char)

check_num(0, "\uFFFD")
check_num(9, "\t")
check_num(1_000_000_000_000_000_000, "\uFFFD")

for entity in ["&quot;;", "&#34;;", "&#x22;;", "&#X22;;"]:
    check(entity, '";')

for entity in ["&quot;quot;", "&#34;quot;", "&#x22;quot;", "&#X22;quot;"]:
    check(entity, '"quot;')

for entity in ["&quot", "&#34", "&#x22", "&#X22"]:
    check(entity * 3, '"""')
    check((entity + ";") * 3, '"""')

for entity in ["&amp", "&amp;", "&AMP", "&AMP;"]:
    check(entity, "&")

for entity in ["&Amp", "&Amp;"]:
    check(entity, entity)

check("&svadilfari;", "&svadilfari;")
check("&notit", "\u00acit")
check("&notit;", "\u00acit;")
check("&notin", "\u00acin")
check("&notin;", "\u2209")
check(
    "&notReallyAnExistingNamedCharacterReference;",
    "\u00acReallyAnExistingNamedCharacterReference;",
)
check("&CounterClockwiseContourIntegral;", "\u2233")
check("&acE;", "\u223e\u0333")
check("&acE", "&acE")
check("&#123; " * 1050, "{ " * 1050)
check(
    "&Eacuteric&Eacute;ric&alphacentauri&alpha;centauri",
    "\u00c9ric\u00c9ric&alphacentauri\u03b1centauri",
)
check("&co;", "&co;")

print("HtmlTests::test_unescape: ok")
