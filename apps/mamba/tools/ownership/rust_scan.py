"""Small deterministic Rust scanner for MbObject collection constructors.

This is deliberately a scanner, not a Rust parser.  It preserves byte offsets
while masking comments and literals, and it refuses incomplete delimiter
sequences instead of guessing.
"""

from __future__ import annotations

from dataclasses import dataclass
import re
from pathlib import Path


CONSTRUCTORS = (
    "new_list_inline_untracked",
    "new_list_borrowed",
    "new_tuple_borrowed",
    "new_set_borrowed",
    "new_list_untracked",
    "new_list_inline",
    "new_list",
    "new_tuple",
    "new_set",
)
CALL_RE = re.compile(
    r"\bMbObject::(" + "|".join(map(re.escape, CONSTRUCTORS)) + r")\b"
)
FN_RE = re.compile(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)[^;{]*\{", re.S)
RAW_STRING_RE = re.compile(r'(?:br|rb|r)(?P<hash>#{0,255})"')


class ScanError(ValueError):
    pass


@dataclass(frozen=True)
class FunctionSpan:
    name: str
    start: int
    body_start: int
    end: int


@dataclass(frozen=True)
class Call:
    constructor: str
    start: int
    end: int
    argument_start: int
    argument_end: int
    symbol: str
    function_start: int
    line: int


def _spaces(text: str) -> str:
    return "".join("\n" if char == "\n" else " " for char in text)


def mask_non_code(source: str, *, literals: bool = True) -> str:
    """Mask comments and, by default, string/character literals."""
    out = list(source)
    i = 0
    size = len(source)
    while i < size:
        if source.startswith("//", i):
            end = source.find("\n", i)
            end = size if end < 0 else end
            out[i:end] = _spaces(source[i:end])
            i = end
            continue
        if source.startswith("/*", i):
            start = i
            depth = 1
            i += 2
            while i < size and depth:
                if source.startswith("/*", i):
                    depth += 1
                    i += 2
                elif source.startswith("*/", i):
                    depth -= 1
                    i += 2
                else:
                    i += 1
            if depth:
                raise ScanError(f"unterminated block comment at byte {start}")
            out[start:i] = _spaces(source[start:i])
            continue

        raw = RAW_STRING_RE.match(source, i)
        if raw:
            start = i
            marker = '"' + raw.group("hash")
            i = raw.end()
            end = source.find(marker, i)
            if end < 0:
                raise ScanError(f"unterminated raw string at byte {start}")
            i = end + len(marker)
            if literals:
                out[start:i] = _spaces(source[start:i])
            continue

        quote_at = i
        if source.startswith('b"', i):
            quote_at = i + 1
        if source[quote_at : quote_at + 1] == '"':
            start = i
            i = quote_at + 1
            escaped = False
            while i < size:
                char = source[i]
                if char == '"' and not escaped:
                    i += 1
                    break
                escaped = char == "\\" and not escaped
                if char != "\\":
                    escaped = False
                i += 1
            else:
                raise ScanError(f"unterminated string at byte {start}")
            if literals:
                out[start:i] = _spaces(source[start:i])
            continue

        # A Rust lifetime (`'a`) has no closing quote; only mask a plausible
        # character literal whose closing quote is nearby.
        if source[i] == "'":
            closing = i + 2
            if i + 1 < size and source[i + 1] == "\\":
                closing = i + 3
                if source[i + 2 : i + 3] == "u":
                    brace = source.find("}", i + 3, min(size, i + 16))
                    closing = brace + 1 if brace >= 0 else closing
            if closing < size and source[closing] == "'":
                end = closing + 1
                if literals:
                    out[i:end] = " " * (end - i)
                i = end
                continue
        i += 1
    return "".join(out)


PAIRS = {"(": ")", "[": "]", "{": "}"}
CLOSERS = {value: key for key, value in PAIRS.items()}


def matching_delimiter(masked: str, opening: int) -> int:
    if opening >= len(masked) or masked[opening] not in PAIRS:
        raise ScanError(f"expected opening delimiter at byte {opening}")
    stack = [masked[opening]]
    for index in range(opening + 1, len(masked)):
        char = masked[index]
        if char in PAIRS:
            stack.append(char)
        elif char in CLOSERS:
            if not stack or stack[-1] != CLOSERS[char]:
                raise ScanError(f"mismatched delimiter at byte {index}")
            stack.pop()
            if not stack:
                return index
    raise ScanError(f"truncated delimiter sequence at byte {opening}")


def split_top_level(text: str) -> list[str]:
    masked = mask_non_code(text)
    stack: list[str] = []
    pieces: list[str] = []
    start = 0
    for index, char in enumerate(masked):
        if char in PAIRS:
            stack.append(char)
        elif char in CLOSERS:
            if not stack or stack[-1] != CLOSERS[char]:
                raise ScanError(f"mismatched delimiter in expression at byte {index}")
            stack.pop()
        elif char == "," and not stack:
            pieces.append(text[start:index].strip())
            start = index + 1
    if stack:
        raise ScanError("truncated expression")
    tail = text[start:].strip()
    if tail:
        pieces.append(tail)
    return pieces


def function_spans(masked: str) -> list[FunctionSpan]:
    spans: list[FunctionSpan] = []
    for match in FN_RE.finditer(masked):
        opening = match.end() - 1
        try:
            end = matching_delimiter(masked, opening)
        except ScanError:
            continue
        spans.append(FunctionSpan(match.group(1), match.start(), opening, end))
    return spans


def scan_calls(source: str) -> tuple[list[Call], list[str]]:
    try:
        masked = mask_non_code(source)
    except ScanError as error:
        return [], [str(error)]
    spans = function_spans(masked)
    calls: list[Call] = []
    diagnostics: list[str] = []
    for match in CALL_RE.finditer(masked):
        cursor = match.end()
        while cursor < len(masked) and masked[cursor].isspace():
            cursor += 1
        if cursor >= len(masked) or masked[cursor] != "(":
            continue
        try:
            closing = matching_delimiter(masked, cursor)
        except ScanError as error:
            diagnostics.append(str(error))
            continue
        owners = [span for span in spans if span.body_start < match.start() < span.end]
        owner = min(owners, key=lambda span: span.end - span.start) if owners else None
        calls.append(
            Call(
                constructor=match.group(1),
                start=match.start(),
                end=closing + 1,
                argument_start=cursor + 1,
                argument_end=closing,
                symbol=owner.name if owner else "<module>",
                function_start=owner.start if owner else 0,
                line=source.count("\n", 0, match.start()) + 1,
            )
        )
    return calls, diagnostics


def rust_files(root: Path) -> list[Path]:
    if root.is_file():
        return [root]
    return sorted(path for path in root.rglob("*.rs") if path.is_file())
