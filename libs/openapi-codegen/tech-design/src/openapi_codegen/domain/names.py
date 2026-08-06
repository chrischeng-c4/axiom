from __future__ import annotations


def words(text: str) -> tuple[str, ...]:
    emitted: list[str] = []
    current: list[str] = []
    prev_lower = False

    for ch in text:
        if ch.isalnum():
            if ch.isupper() and prev_lower and current:
                emitted.append("".join(current))
                current = []
            current.append(ch)
            prev_lower = ch.islower() or ch.isnumeric()
        else:
            if current:
                emitted.append("".join(current))
                current = []
            prev_lower = False

    if current:
        emitted.append("".join(current))

    return tuple(emitted)


def capitalize(word: str) -> str:
    if not word:
        return ""
    return word[0].upper() + word[1:].lower()


def to_pascal(text: str) -> str:
    s = "".join(capitalize(w) for w in words(text))
    if s == "":
        s = "Anonymous"
    if not s[0].isalpha():
        s = "_" + s
    return s


def to_camel(text: str) -> str:
    p = to_pascal(text)
    return p[0].lower() + p[1:]


def to_snake(text: str) -> str:
    s = "_".join(w.lower() for w in words(text))
    if s == "":
        return "field"
    if not s[0].isalpha() and s[0] != "_":
        return "_" + s
    return s


def is_ident(text: str) -> bool:
    if not text:
        return False
    c0 = text[0]
    if not (("a" <= c0 <= "z") or ("A" <= c0 <= "Z") or c0 == "_" or c0 == "$"):
        return False
    for ch in text[1:]:
        if not (
            ("a" <= ch <= "z")
            or ("A" <= ch <= "Z")
            or ("0" <= ch <= "9")
            or ch == "_"
            or ch == "$"
        ):
            return False
    return True


def escape_string_literal(text: str) -> str:
    return text.replace("\\", "\\\\").replace('"', '\\"')


def prop_key(key: str) -> str:
    if is_ident(key):
        return key
    return '"' + escape_string_literal(key) + '"'


def param_access(name: str) -> str:
    if is_ident(name):
        return "params." + name
    return 'params["' + escape_string_literal(name) + '"]'


class NameRegistry:
    def __init__(self) -> None:
        self._used: set[str] = set()

    def unique(self, base: str) -> str:
        if base == "":
            base = "anonymous"
        if base not in self._used:
            self._used.add(base)
            return base
        n = 2
        while True:
            candidate = f"{base}_{n}"
            if candidate not in self._used:
                self._used.add(candidate)
                return candidate
            n += 1

    def taken(self) -> frozenset[str]:
        return frozenset(self._used)
