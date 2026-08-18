from __future__ import annotations

import sys
import unittest

sys.path.insert(0, __file__.rsplit("/", 3)[0] + "/src")

from openapi_codegen.domain.names import (
    NameRegistry,
    capitalize,
    escape_string_literal,
    is_ident,
    param_access,
    prop_key,
    to_camel,
    to_pascal,
    to_snake,
    words,
)


class TestDomainNames(unittest.TestCase):
    def test_words_capital_run(self) -> None:
        # Tell 1: run of capitals is ONE word
        self.assertEqual(words("HTTPServer"), ("HTTPServer",))
        # Paired test: lower-to-upper split
        self.assertEqual(words("HttpServer"), ("Http", "Server"))

    def test_words_digit_split(self) -> None:
        # Tell 2: digit sets prev_lower, so following capital splits
        self.assertEqual(words("v2Api"), ("v2", "Api"))
        # Paired test: letter before capital
        self.assertEqual(words("vApi"), ("v", "Api"))

    def test_words_hump_split(self) -> None:
        self.assertEqual(words("petId"), ("pet", "Id"))

    def test_words_trailing_capitals(self) -> None:
        self.assertEqual(words("petID"), ("pet", "ID"))

    def test_words_separators_and_empty(self) -> None:
        self.assertEqual(words("--pet__category--"), ("pet", "category"))
        self.assertEqual(words(""), ())

    def test_capitalize(self) -> None:
        self.assertEqual(capitalize("ID"), "Id")
        self.assertEqual(capitalize("pet"), "Pet")
        self.assertEqual(capitalize(""), "")
        self.assertEqual(capitalize("123abc"), "123abc")

    def test_to_pascal(self) -> None:
        # Tell 3: to_pascal("petID") == "PetId"
        self.assertEqual(to_pascal("petID"), "PetId")
        self.assertEqual(to_pascal("pet_category"), "PetCategory")
        self.assertEqual(to_pascal("pet-category"), "PetCategory")
        # Tell 4: placeholder for empty string
        self.assertEqual(to_pascal(""), "Anonymous")
        self.assertEqual(to_pascal("---"), "Anonymous")
        self.assertEqual(to_pascal("123abc"), "_123abc")

    def test_to_camel(self) -> None:
        # Tell 4: placeholder for empty string in to_camel is "anonymous"
        self.assertEqual(to_camel(""), "anonymous")
        self.assertEqual(to_camel("123abc"), "_123abc")
        self.assertEqual(to_camel("PetCategory"), "petCategory")
        self.assertEqual(to_camel("list_pets"), "listPets")

    def test_to_snake(self) -> None:
        # Tell 4: placeholder for empty string in to_snake is "field"
        self.assertEqual(to_snake(""), "field")
        self.assertEqual(to_snake("HTTPServer"), "httpserver")
        self.assertEqual(to_snake("petId"), "pet_id")
        self.assertEqual(to_snake("123abc"), "_123abc")

    def test_is_ident_valid(self) -> None:
        # Tell 5: '$' is legal as first and subsequent char
        self.assertTrue(is_ident("$ref"))
        self.assertTrue(is_ident("_private"))
        self.assertTrue(is_ident("foo123"))
        self.assertTrue(is_ident("A$B_1"))

    def test_is_ident_invalid_ascii(self) -> None:
        # Tell 6: non-ASCII characters fail even if str.isalpha() is True
        self.assertFalse(is_ident("café"))
        self.assertFalse(is_ident("X-Request-Id"))
        self.assertFalse(is_ident(""))
        self.assertFalse(is_ident("123abc"))

    def test_escape_string_literal_order(self) -> None:
        # Backslash is replaced FIRST, then the double quote. The input is the
        # seven characters: a, space, backslash, space, quote, space, b.
        inp = 'a \\ " b'
        self.assertEqual(len(inp), 7)
        out = escape_string_literal(inp)
        self.assertEqual(len(out), 9)
        self.assertEqual(
            tuple(out),
            ("a", " ", "\\", "\\", " ", "\\", '"', " ", "b"),
        )
        # Quote-first would have escaped the backslash introduced by the quote
        # replacement, yielding a doubled backslash before the quote.
        self.assertNotEqual(
            tuple(out), ("a", " ", "\\", "\\", " ", "\\", "\\", '"', " ", "b")
        )

    def test_escape_string_literal_backslash_before_quote(self) -> None:
        # A single backslash becomes two; a single quote becomes backslash-quote.
        self.assertEqual(escape_string_literal("\\"), "\\\\")
        self.assertEqual(escape_string_literal('"'), '\\"')
        # The composite case is the discriminator: backslash-first gives three
        # backslashes then a quote; quote-first would give four then a quote.
        self.assertEqual(escape_string_literal('\\"'), '\\\\\\"')
        self.assertEqual(len(escape_string_literal('\\"')), 4)

    def test_prop_key(self) -> None:
        self.assertEqual(prop_key("foo"), "foo")
        self.assertEqual(prop_key("$ref"), "$ref")
        self.assertEqual(prop_key("x-key"), '"x-key"')
        self.assertEqual(prop_key('a\\"b'), '"a\\\\\\"b"')

    def test_param_access(self) -> None:
        self.assertEqual(param_access("foo"), "params.foo")
        self.assertEqual(param_access("$ref"), "params.$ref")
        self.assertEqual(param_access("x-key"), 'params["x-key"]')

    def test_name_registry_unique_sequence(self) -> None:
        reg = NameRegistry()
        self.assertEqual(reg.unique("Pet"), "Pet")
        self.assertEqual(reg.unique("Pet"), "Pet_2")
        self.assertEqual(reg.unique("Pet"), "Pet_3")

    def test_name_registry_explicit_registration_collision(self) -> None:
        # Tell 8: after unique("Pet") and unique("Pet_2"), next unique("Pet") returns "Pet_3"
        reg = NameRegistry()
        self.assertEqual(reg.unique("Pet"), "Pet")
        self.assertEqual(reg.unique("Pet_2"), "Pet_2")
        self.assertEqual(reg.unique("Pet"), "Pet_3")

    def test_name_registry_anonymous_placeholders(self) -> None:
        # Tell 4: NameRegistry().unique("") returns "anonymous"
        reg = NameRegistry()
        self.assertEqual(reg.unique(""), "anonymous")
        self.assertEqual(reg.unique(""), "anonymous_2")

    def test_name_registry_taken(self) -> None:
        reg = NameRegistry()
        reg.unique("Pet")
        reg.unique("User")
        taken_names = reg.taken()
        self.assertIsInstance(taken_names, frozenset)
        self.assertEqual(taken_names, frozenset({"Pet", "User"}))


if __name__ == "__main__":
    unittest.main()
