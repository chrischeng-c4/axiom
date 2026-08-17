from __future__ import annotations


def parse_line_ranges(range_spec: str) -> list[tuple[int, int, int]] | str:
    if not isinstance(range_spec, str):
        raise TypeError("line range must be a string")
    spec_clean = range_spec.strip()
    if not spec_clean:
        raise ValueError("empty range specification")
    if spec_clean.lower() in ("any", "new"):
        return spec_clean.lower()
    ranges = []
    for item in spec_clean.split(","):
        item_str = item.strip()
        if not item_str:
            raise ValueError("empty range segment")
        if "~" in item_str:
            range_part, slack_part = item_str.split("~", 1)
            if "~" in slack_part:
                raise ValueError("multiple ~ in range segment")
            try:
                slack = int(slack_part.strip())
            except ValueError:
                raise ValueError(f"invalid slack value: {slack_part}")
            if slack < 0:
                raise ValueError(f"negative slack: {slack}")
        else:
            range_part = item_str
            slack = 0

        range_part = range_part.strip()
        if "-" in range_part:
            start_str, end_str = range_part.split("-", 1)
            if "-" in end_str:
                raise ValueError(f"invalid range format: {range_part}")
            try:
                start = int(start_str.strip())
                end = int(end_str.strip())
            except ValueError:
                raise ValueError(f"invalid range bounds: {range_part}")
            if end < start:
                raise ValueError(f"range end {end} precedes start {start}")
        else:
            try:
                start = int(range_part)
            except ValueError:
                raise ValueError(f"invalid range number: {range_part}")
            end = start

        ranges.append((start, end, slack))
    return ranges
