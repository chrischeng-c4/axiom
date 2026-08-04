from __future__ import annotations


def release_tag(
    project: str, version: str | None, fallback_version: str
) -> str:
    chosen = (
        version
        if version is not None and version.strip() != ""
        else fallback_version
    )
    v_str = chosen.strip()
    prefix = f"{project}@"
    if v_str.startswith(prefix):
        return v_str
    return f"{prefix}{v_str}"


def strip_source_ownership_markers(text: str) -> str:
    if not text:
        return ""
    lines = text.splitlines()
    kept: list[str] = []
    for line in lines:
        lstripped = line.lstrip()
        if (
            lstripped.startswith("# SPEC-MANAGED:")
            or lstripped == "# CODEGEN-BEGIN"
            or lstripped == "# CODEGEN-END"
        ):
            continue
        kept.append(line)
    if not kept:
        return ""
    return "\n".join(kept) + "\n"


def replace_kubernetes_namespace(
    text: str, checked_in_namespace: str, namespace: str
) -> str:
    res = text.replace(
        f"name: {checked_in_namespace}", f"name: {namespace}"
    )
    return res.replace(
        f"namespace: {checked_in_namespace}", f"namespace: {namespace}"
    )


def ensure_trailing_newline(text: str) -> str:
    if text.endswith("\n"):
        return text
    return text + "\n"
