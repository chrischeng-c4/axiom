from __future__ import annotations

from service_http.domain.trace import (
    DEFAULT_TRACE_FLAGS,
    HYPHEN_POSITIONS,
    PARENT_SPAN_ID_SPAN,
    SUPPORTED_VERSION,
    TRACE_FLAGS_SPAN,
    TRACE_ID_SPAN,
    TRACEPARENT_LENGTH,
    VERSION_SPAN,
    TraceContext,
    TraceParent,
    is_all_zero,
    is_lower_hex,
)


def parse_traceparent(values: tuple[str, ...]) -> TraceParent | None:
    if len(values) != 1:
        return None
    value = values[0]
    if any(ord(c) > 127 for c in value):
        return None
    if len(value) != TRACEPARENT_LENGTH:
        return None
    for pos in HYPHEN_POSITIONS:
        if value[pos] != "-":
            return None
    version = value[VERSION_SPAN[0] : VERSION_SPAN[1]]
    trace_id = value[TRACE_ID_SPAN[0] : TRACE_ID_SPAN[1]]
    parent_span_id = value[PARENT_SPAN_ID_SPAN[0] : PARENT_SPAN_ID_SPAN[1]]
    trace_flags = value[TRACE_FLAGS_SPAN[0] : TRACE_FLAGS_SPAN[1]]

    if version != SUPPORTED_VERSION:
        return None
    if not is_lower_hex(trace_id):
        return None
    if not is_lower_hex(parent_span_id):
        return None
    if not is_lower_hex(trace_flags):
        return None
    if is_all_zero(trace_id):
        return None
    if is_all_zero(parent_span_id):
        return None
    return TraceParent(version, trace_id, parent_span_id, trace_flags)


def request_trace_context(
    values: tuple[str, ...],
    fresh_trace_id: str,
    fresh_span_id: str,
) -> TraceContext:
    parsed = parse_traceparent(values)
    if parsed is None:
        return TraceContext(
            trace_id=fresh_trace_id,
            span_id=fresh_span_id,
            parent_span_id=None,
            trace_flags=DEFAULT_TRACE_FLAGS,
        )
    return TraceContext(
        trace_id=parsed.trace_id,
        span_id=fresh_span_id,
        parent_span_id=parsed.parent_span_id,
        trace_flags=parsed.trace_flags,
    )


def span_fields(context: TraceContext) -> dict[str, object]:
    body: dict[str, object] = {
        "trace_id": context.trace_id,
        "span_id": context.span_id,
        "trace_flags": context.trace_flags,
    }
    if context.parent_span_id is not None:
        body["parent_span_id"] = context.parent_span_id
    return body
