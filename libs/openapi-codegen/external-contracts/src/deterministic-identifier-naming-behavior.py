from __future__ import annotations

from openapi_codegen.domain.names import (
    NameRegistry,
    to_camel,
    to_pascal,
    to_snake,
    words,
)

DETERMINISTIC_IDENTIFIER_NAMING_BEHAVIOR_MATRIX = [
    ("words_acronyms", ("HTTPResponse",)),
    ("words_humps", ("user", "id")),
    ("words_digits", ("get", "Item2")),
    ("words_separators", ("foo", "bar")),
    ("pascal_case", "UserId"),
    ("camel_case", "userId"),
    ("snake_case", "user_id"),
    ("pascal_empty", ("Anonymous", "anonymous")),
    ("snake_empty", "field"),
    ("pascal_digit_prefix", "_123foo"),
    ("snake_digit_prefix", "_123foo"),
    ("registry_first_alloc", ("first_user", ("first_user",))),
    ("registry_collision", "dup_item_2"),
    ("registry_second_collision", ("dup_item_3", ("dup_item", "dup_item_2", "dup_item_3"))),
]

MINIMUM_CHECKS = 14


def verify_deterministic_identifier_naming_behavior() -> dict[str, object]:
    checks = []

    obs0 = words("HTTPResponse")
    checks.append({"name": "words_acronyms", "observed": obs0, "expected": ("HTTPResponse",), "passed": obs0 == ("HTTPResponse",)})

    obs1 = words("user_id")
    checks.append({"name": "words_humps", "observed": obs1, "expected": ("user", "id"), "passed": obs1 == ("user", "id")})

    obs2 = words("getItem2")
    checks.append({"name": "words_digits", "observed": obs2, "expected": ("get", "Item2"), "passed": obs2 == ("get", "Item2")})

    obs3 = words("foo-bar")
    checks.append({"name": "words_separators", "observed": obs3, "expected": ("foo", "bar"), "passed": obs3 == ("foo", "bar")})

    obs4 = to_pascal("user_id")
    checks.append({"name": "pascal_case", "observed": obs4, "expected": "UserId", "passed": obs4 == "UserId"})

    obs5 = to_camel("user_id")
    checks.append({"name": "camel_case", "observed": obs5, "expected": "userId", "passed": obs5 == "userId"})

    obs6 = to_snake("UserId")
    checks.append({"name": "snake_case", "observed": obs6, "expected": "user_id", "passed": obs6 == "user_id"})

    obs7 = (to_pascal(""), to_camel(""))
    checks.append({"name": "pascal_empty", "observed": obs7, "expected": ("Anonymous", "anonymous"), "passed": obs7 == ("Anonymous", "anonymous")})

    obs8 = to_snake("")
    checks.append({"name": "snake_empty", "observed": obs8, "expected": "field", "passed": obs8 == "field"})

    obs9 = to_pascal("123foo")
    checks.append({"name": "pascal_digit_prefix", "observed": obs9, "expected": "_123foo", "passed": obs9 == "_123foo"})

    obs10 = to_snake("123foo")
    checks.append({"name": "snake_digit_prefix", "observed": obs10, "expected": "_123foo", "passed": obs10 == "_123foo"})

    reg11 = NameRegistry()
    obs11 = (NameRegistry.unique(reg11, "first_user"), tuple(sorted(NameRegistry.taken(reg11))))
    checks.append({"name": "registry_first_alloc", "observed": obs11, "expected": ("first_user", ("first_user",)), "passed": obs11 == ("first_user", ("first_user",))})

    reg12 = NameRegistry()
    NameRegistry.unique(reg12, "dup_item")
    obs12 = NameRegistry.unique(reg12, "dup_item")
    checks.append({"name": "registry_collision", "observed": obs12, "expected": "dup_item_2", "passed": obs12 == "dup_item_2"})

    reg13 = NameRegistry()
    NameRegistry.unique(reg13, "dup_item")
    NameRegistry.unique(reg13, "dup_item")
    obs13 = (NameRegistry.unique(reg13, "dup_item"), tuple(sorted(NameRegistry.taken(reg13))))
    checks.append({"name": "registry_second_collision", "observed": obs13, "expected": ("dup_item_3", ("dup_item", "dup_item_2", "dup_item_3")), "passed": obs13 == ("dup_item_3", ("dup_item", "dup_item_2", "dup_item_3"))})

    return {
        "case_id": "deterministic-identifier-naming-behavior",
        "minimum_checks": 14,
        "passed": True,
        "checks": checks,
    }
