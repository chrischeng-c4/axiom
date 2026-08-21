# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "real_world"
# case = "rss_feed_extraction"
# subject = "ET"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""ET: a feed-reader job parses a synthetic RSS document, iterates the <item> entries, extracts title/link/pubDate text, aggregates a count and a deterministic title digest, and re-serializes one filtered item"""
import xml.etree.ElementTree as ET

# Synthetic RSS 2.0 feed. Deterministic per-item variation so the parser
# actually has to walk the document.
N = 50
items_xml = []
for i in range(N):
    items_xml.append(
        "<item>"
        f"<title>Post {i:03d}</title>"
        f"<link>https://example.com/p/{i}</link>"
        f"<pubDate>01 May 2026 {i % 24:02d}:00:00 GMT</pubDate>"
        f"<category>{('news', 'blog', 'release')[i % 3]}</category>"
        "</item>"
    )
doc = (
    '<?xml version="1.0" encoding="UTF-8"?>'
    '<rss version="2.0"><channel>'
    "<title>Example Feed</title>"
    "<link>https://example.com/</link>"
    + "".join(items_xml)
    + "</channel></rss>"
)

root = ET.fromstring(doc)
channel = root.find("channel")
assert channel is not None, "channel present"
assert channel.findtext("title") == "Example Feed", "channel title"

items = channel.findall("item")
assert len(items) == N, f"item count = {len(items)!r}"

# Extract title/link/pubDate and aggregate a deterministic title digest.
title_digest = 0
release_count = 0
for it in items:
    title = it.findtext("title")
    link = it.findtext("link")
    pub = it.findtext("pubDate")
    assert title.startswith("Post "), f"title shape: {title!r}"
    assert link.startswith("https://example.com/p/"), f"link shape: {link!r}"
    assert pub.endswith(" GMT"), f"pubDate shape: {pub!r}"
    title_digest += sum(ord(ch) for ch in title)
    if it.findtext("category") == "release":
        release_count += 1

assert title_digest > 0, "title digest accumulated"
# categories cycle news/blog/release, so 'release' appears every 3rd item.
assert release_count == sum(1 for i in range(N) if i % 3 == 2), "release count matches cycle"

# Re-serialize one filtered item and confirm it round-trips.
first = items[0]
s = ET.tostring(first, encoding="unicode")
again = ET.fromstring(s)
assert again.findtext("title") == "Post 000", "round-trip filtered item title"

print("rss_feed_extraction OK")
