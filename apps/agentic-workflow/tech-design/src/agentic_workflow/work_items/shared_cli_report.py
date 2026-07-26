"""Shared CLI issue intake contract.

@spec #2597
"""

from __future__ import annotations


def canonical_report_labels(app_name: str, caller_labels: tuple[str, ...]) -> tuple[str, ...]:
    """Preserve caller labels and add the typed intake identity exactly once."""

    labels = list(caller_labels)
    for required in (f"app:{app_name}", "type:report"):
        if required not in labels:
            labels.append(required)
    return tuple(labels)
