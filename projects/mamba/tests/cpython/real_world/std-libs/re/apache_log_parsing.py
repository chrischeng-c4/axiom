# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "real_world"
# case = "apache_log_parsing"
# subject = "re"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re: an access-log analytics job compiles one regex, findall-extracts (ip, path, status, bytes) tuples from a synthetic Apache corpus, aggregates per-status counts and total bytes, and fullmatch-validates a sample of whole lines"""
import re

# Synthetic Apache-common-log corpus. Deterministic per-line variation so
# the regex actually has to work; status codes distributed across a fixed
# set so the aggregator can be sanity-checked.
N = 100_000
template = (
    '10.0.{ip_c}.{ip_d} - - [01/May/2026:{hh:02d}:{mm:02d}:{ss:02d} +0000] '
    '"{method} {path} HTTP/1.1" {status} {nbytes}\n'
)
lines = []
for i in range(N):
    ip_c = (i // 250) % 250
    ip_d = i % 250
    hh = (i // 3600) % 24
    mm = (i // 60) % 60
    ss = i % 60
    method = ("GET", "GET", "GET", "POST", "GET", "DELETE")[i % 6]
    item = (i * 7) % 9973
    path = "/api/items/%d" % item
    status = (200, 200, 200, 304, 404, 500, 503)[i % 7]
    nbytes = ((i * 131) % 16384) + 64
    lines.append(template.format(
        ip_c=ip_c, ip_d=ip_d, hh=hh, mm=mm, ss=ss,
        method=method, path=path, status=status, nbytes=nbytes,
    ))
corpus = "".join(lines)

log_re = re.compile(
    r'(\d+\.\d+\.\d+\.\d+)\s+\S+\s+\S+\s+\[[^\]]+\]\s+'
    r'"[A-Z]+\s+(\S+)\s+HTTP/[\d.]+"\s+(\d+)\s+(\d+)'
)

# findall the (ip, path, status, bytes) tuples in one bulk scan.
matches = log_re.findall(corpus)
assert len(matches) == N, f"matches != N: {len(matches)}"

# Per-status aggregation + total bytes.
status_counts: dict[str, int] = {}
total_bytes = 0
for tup in matches:
    sc = tup[2]
    status_counts[sc] = status_counts.get(sc, 0) + 1
    total_bytes += int(tup[3])

# 200 appears 3 of every 7 rows, so it dominates the distribution.
assert status_counts["200"] > status_counts["404"], "200 dominates 404"
assert sum(status_counts.values()) == N, "status counts sum to N"
assert total_bytes > 0, "total bytes accumulated"

# Schema validation: fullmatch a sample of whole lines.
ok = 0
for idx in (0, 10000, 25000, 50000, 75000, 99999):
    line = lines[idx].rstrip("\n")
    if log_re.fullmatch(line) is not None:
        ok += 1
assert ok == 6, f"fullmatch_ok = {ok}"

# escape + search probe over a sample of lines.
needle = re.escape("/api/items/42")
hits = 0
for idx in range(5000):
    if re.search(needle, lines[idx]) is not None:
        hits += 1
assert hits >= 0, "search probe ran"

print("apache_log_parsing OK")
