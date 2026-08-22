use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/html_parser/convert_charrefs_false_bad_entity_no_raise.py`.
#[test]
fn test_gen_errors_std_libs_html_parser_convert_charrefs_false_bad_entity_no_raise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "errors"
# case = "convert_charrefs_false_bad_entity_no_raise"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: with convert_charrefs=False, feeding a non-entity (&not_an_entity;) does not raise"""
from html.parser import HTMLParser

p = HTMLParser(convert_charrefs=False)
# A non-entity ampersand sequence must be tolerated, not raised on.
p.feed("&not_an_entity;")
p.close()
print("convert_charrefs_false_bad_entity_no_raise OK")
"###);
    assert_output(&out, r###"convert_charrefs_false_bad_entity_no_raise OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/html_parser/get_starttag_text_none_when_no_tag.py`.
#[test]
fn test_gen_errors_std_libs_html_parser_get_starttag_text_none_when_no_tag() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "errors"
# case = "get_starttag_text_none_when_no_tag"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: get_starttag_text() returns None on a fresh parser with no current start tag"""
from html.parser import HTMLParser

p = HTMLParser()
assert p.get_starttag_text() is None, p.get_starttag_text()
print("get_starttag_text_none_when_no_tag OK")
"###);
    assert_output(&out, r###"get_starttag_text_none_when_no_tag OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/html_parser/malformed_input_does_not_raise.py`.
#[test]
fn test_gen_errors_std_libs_html_parser_malformed_input_does_not_raise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "errors"
# case = "malformed_input_does_not_raise"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: the default parser is forgiving: feeding an unclosed tag (<html><body><unclosed_tag>text) records what it can and does NOT raise"""
from html.parser import HTMLParser


class CaptureParser(HTMLParser):
    def __init__(self):
        super().__init__()
        self.tags = []

    def handle_starttag(self, tag, attrs):
        self.tags.append(tag)


cp = CaptureParser()
# A forgiving lexer: this must not raise; it records what start tags it can.
cp.feed("<html><body><unclosed_tag>text")
assert cp.tags == ["html", "body", "unclosed_tag"], cp.tags
print("malformed_input_does_not_raise OK")
"###);
    assert_output(&out, r###"malformed_input_does_not_raise OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/html_parser/subclass_handler_exception_propagates.py`.
#[test]
fn test_gen_errors_std_libs_html_parser_subclass_handler_exception_propagates() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "errors"
# case = "subclass_handler_exception_propagates"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: an exception raised inside a subclass handle_starttag override propagates out of feed() (ValueError on a <script> tag)"""
from html.parser import HTMLParser


class StrictParser(HTMLParser):
    def handle_starttag(self, tag, attrs):
        if tag == "script":
            raise ValueError("scripts not allowed")


_raised = False
try:
    StrictParser().feed("<html><body><script>bad</script></body></html>")
except ValueError:
    _raised = True
assert _raised, "subclass handle_starttag ValueError must propagate out of feed()"
print("subclass_handler_exception_propagates OK")
"###);
    assert_output(&out, r###"subclass_handler_exception_propagates OK
"###);
}
