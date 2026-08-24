"""Local provenance classification for ownership-taking constructors."""

from __future__ import annotations

from dataclasses import dataclass
import re

from rust_scan import ScanError, mask_non_code, matching_delimiter, split_top_level


OWNED = "OWNED"
BORROWED = "BORROWED"
MIXED = "MIXED"
UNCLASSIFIED = "UNCLASSIFIED"


@dataclass(frozen=True)
class Provenance:
    classification: str
    fingerprint: str
    evidence: str


def normalize(expression: str) -> str:
    return re.sub(r"\s+", "", expression)


def combine(parts: list[Provenance], prefix: str) -> Provenance:
    if not parts:
        return Provenance(OWNED, f"{prefix}[]", "empty collection")
    classes = {part.classification for part in parts}
    if UNCLASSIFIED in classes:
        classification = UNCLASSIFIED
    elif classes == {OWNED}:
        classification = OWNED
    elif classes == {BORROWED}:
        classification = BORROWED
    else:
        classification = MIXED
    fingerprints = ",".join(part.fingerprint for part in parts)
    return Provenance(classification, f"{prefix}[{fingerprints}]", "element provenance")


def _latest_binding(prefix: str, name: str) -> tuple[str, int] | None:
    masked = mask_non_code(prefix)
    pattern = re.compile(
        r"\blet\s+(?:mut\s+)?"
        + re.escape(name)
        + r"(?:\s*:\s*[^=;]+)?\s*=",
        re.S,
    )
    matches = list(pattern.finditer(masked))
    if not matches:
        return None
    match = matches[-1]
    start = match.end()
    stack: list[str] = []
    pairs = {"(": ")", "[": "]", "{": "}"}
    closers = {value: key for key, value in pairs.items()}
    for index in range(start, len(masked)):
        char = masked[index]
        if char in pairs:
            stack.append(char)
        elif char in closers:
            if stack and stack[-1] == closers[char]:
                stack.pop()
            elif not stack:
                break
        elif char == ";" and not stack:
            return prefix[start:index].strip(), index + 1
    return None


def _push_values(prefix: str, name: str, after: int) -> list[str]:
    masked = mask_non_code(prefix)
    pattern = re.compile(r"\b" + re.escape(name) + r"\s*\.\s*push\s*\(")
    values: list[str] = []
    for match in pattern.finditer(masked, after):
        opening = masked.find("(", match.start(), match.end())
        depth = 1
        index = opening + 1
        while index < len(masked) and depth:
            if masked[index] == "(":
                depth += 1
            elif masked[index] == ")":
                depth -= 1
            index += 1
        if depth:
            raise ScanError(f"truncated push for {name}")
        values.append(prefix[opening + 1 : index - 1].strip())
    return values


def _function_parameters(prefix: str) -> set[str]:
    header = prefix[: prefix.find("{")] if "{" in prefix else ""
    masked = mask_non_code(header)
    match = re.search(r"\bfn\s+\w+(?:\s*<[^>]*>)?\s*\(", masked, re.S)
    if not match:
        return set()
    opening = match.end() - 1
    try:
        closing = matching_delimiter(masked, opening)
    except ScanError:
        return set()
    result: set[str] = set()
    for item in split_top_level(header[opening + 1 : closing]):
        name = item.split(":", 1)[0].strip()
        name = re.sub(r"^(?:mut|ref)\s+", "", name)
        if re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", name):
            result.add(name)
    return result


def classify(
    expression: str,
    function_prefix: str,
    *,
    constructor: str,
    seen: frozenset[str] = frozenset(),
    known_helpers: frozenset[str] = frozenset(),
) -> Provenance:
    expr = mask_non_code(expression, literals=False).strip()
    norm = normalize(expr)
    if not expr:
        return Provenance(OWNED, "empty", "zero arguments")

    macro = re.fullmatch(r"(?:vec|smallvec)!\s*\[(.*)\]", expr, re.S)
    if macro:
        try:
            members = split_top_level(macro.group(1))
        except ScanError as error:
            return Provenance(UNCLASSIFIED, "truncated", str(error))
        return combine(
            [
                classify(
                    member,
                    function_prefix,
                    constructor=constructor,
                    seen=seen,
                    known_helpers=known_helpers,
                )
                for member in members
            ],
            "elements",
        )

    if re.match(r"(?:Vec|SmallVec|HashSet)::(?:new|with_capacity)\s*\(", expr) or re.fullmatch(
        r"Default::default\s*\(\s*\)", expr, re.S
    ):
        return Provenance(OWNED, "empty-constructor", "fresh empty collection")

    if re.fullmatch(r"(?:true|false|None|Some\([^)]*\)|[-+]?\d+(?:\.\d+)?)", expr, re.S):
        return Provenance(OWNED, f"immediate({norm})", "non-pointer immediate")

    if "retain_if_ptr" in expr or "push_retained" in expr:
        return Provenance(OWNED, f"explicit-retain({norm})", "explicit retain")

    # A newly allocated MbObject pointer, or an evaluator result which transfers
    # an owned value, is fresh at this boundary.
    if re.search(r"\bMbObject::new_[A-Za-z0-9_]+\s*\(", expr):
        return Provenance(OWNED, f"fresh-object({norm})", "nested object constructor")
    if re.search(
        r"\b(?:eval_expr|call_function|invoke_callable|make_ast_node|"
        r"strict_param_sig|timedelta_from_us)\s*\(",
        expr,
    ):
        return Provenance(OWNED, f"owned-return({norm})", "known ownership-returning call")
    if re.search(r"\bMbValue::from_(?:int|float|bool|none)\s*\(", expr):
        return Provenance(OWNED, f"immediate({norm})", "boxed immediate")
    if re.search(
        r"\b(?:MbValue::none|s|b|mk_str|str_val|str_v|s_val|new_str|"
        r"new_bytes|empty_list)\s*\(",
        expr,
    ):
        return Provenance(OWNED, f"owned-value({norm})", "known owned value constructor")

    identifier = re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", expr)
    if identifier:
        name = identifier.group(0)
        if name in seen:
            return Provenance(BORROWED, "rebound-input", f"rebound enclosing input {name}")
        binding = _latest_binding(function_prefix, name)
        if binding:
            rhs, after = binding
            base = classify(
                rhs,
                function_prefix[:after],
                constructor=constructor,
                seen=seen | {name},
                known_helpers=known_helpers,
            )
            try:
                pushes = _push_values(function_prefix, name, after)
            except ScanError as error:
                return Provenance(UNCLASSIFIED, f"binding({base.fingerprint})", str(error))
            if pushes:
                pushed = [
                    classify(
                        value,
                        function_prefix,
                        constructor=constructor,
                        seen=seen | {name},
                        known_helpers=known_helpers,
                    )
                    for value in pushes
                ]
                base = combine(([base] if base.fingerprint != "empty-constructor" else []) + pushed, "built")
            return Provenance(
                base.classification,
                base.fingerprint,
                f"local binding {name}: {base.evidence}",
            )
        if name in _function_parameters(function_prefix):
            return Provenance(BORROWED, "parameter", f"function parameter {name}")
        return Provenance(BORROWED, "external-binding", f"enclosing or closure binding {name}")

    if re.search(r"\.(?:to_vec|clone|copied)\s*\(", expr) or re.search(
        r"\[[^\]]*(?:\.\.)?[^\]]*\]", expr
    ):
        return Provenance(BORROWED, f"pointer-copy({norm})", "container pointer copy")

    collect = re.search(r"\.map\s*\(\s*\|[^|]*\|\s*(.*?)\)\s*\.collect", expr, re.S)
    if collect:
        mapped = classify(
            collect.group(1),
            function_prefix,
            constructor=constructor,
            seen=seen,
            known_helpers=known_helpers,
        )
        return Provenance(mapped.classification, f"collect({mapped.fingerprint})", mapped.evidence)

    if re.fullmatch(r"(?:&?\s*)?[A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)+", expr):
        return Provenance(BORROWED, f"field({norm})", "borrowed field or member")

    if re.match(r"(?:unsafe\s*)?\{\s*\*", expr) or re.match(r"\*", expr):
        return Provenance(BORROWED, f"dereference({norm})", "dereferenced existing value")

    call = re.match(
        r"(?:[A-Za-z_][A-Za-z0-9_]*\.)*([A-Za-z_][A-Za-z0-9_:]*)\s*(?:!?\s*)\(",
        expr,
    )
    if call:
        basename = call.group(1).split("::")[-1]
        if "." in expr[: call.end()]:
            return Provenance(
                BORROWED,
                f"method-result({basename})",
                f"conservative method result {basename}",
            )
        local_callable = re.search(
            r"\blet\s+(?:mut\s+)?"
            + re.escape(basename)
            + r"\s*(?::[^=]+)?=\s*\|",
            function_prefix,
        )
        if (
            basename in known_helpers
            or basename in _function_parameters(function_prefix)
            or local_callable
        ):
            return Provenance(
                BORROWED,
                f"local-helper({basename})",
                f"conservative local helper return {basename}",
            )
        return Provenance(
            UNCLASSIFIED,
            f"opaque-call({call.group(1)})",
            f"opaque helper return {call.group(1)}",
        )

    # Any complete but unsupported Rust expression is conservatively borrowed:
    # this can create a false-positive remediation candidate, never certify an
    # unsafe ownership-taking site as clean. Unknown call targets above remain
    # UNCLASSIFIED because even the value boundary cannot be established.
    return Provenance(BORROWED, f"conservative({norm})", "unsupported complete expression")
