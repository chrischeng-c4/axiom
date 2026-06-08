---
number: 390
title: "feat(mamba): stdlib datetime and time"
state: open
labels: [enhancement, P2, crate:mamba]
---

# #390 — feat(mamba): stdlib datetime and time

## Summary
Implement `datetime` and `time` standard library modules.

## Required (datetime)
- `datetime.datetime(year, month, day, hour, minute, second)`
- `datetime.datetime.now()`, `.utcnow()`, `.today()`
- `datetime.date(year, month, day)`, `datetime.time(hour, minute, second)`
- `datetime.timedelta(days, seconds, ...)`
- Arithmetic: `datetime - datetime → timedelta`, `datetime + timedelta → datetime`
- Formatting: `.strftime(format)`, `.isoformat()`
- Parsing: `datetime.strptime(string, format)`

## Required (time)
- `time.time()` → float (epoch seconds)
- `time.sleep(seconds)`
- `time.monotonic()`

## Implementation Notes
- Use Rust `chrono` crate as backend for datetime
- Use `std::time` for time module
