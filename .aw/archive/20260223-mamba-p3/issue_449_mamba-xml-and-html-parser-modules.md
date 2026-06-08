---
number: 449
title: "mamba: xml and html.parser modules"
state: open
labels: [enhancement, crate:mamba, P3]
---

# #449 — mamba: xml and html.parser modules

## Description

Implement basic XML/HTML parsing modules.

## Requirements

### html.parser
- R1: `html.parser.HTMLParser` — event-based HTML parser
- R2: Override `handle_starttag`, `handle_endtag`, `handle_data`
- R3: `html.escape(s)` / `html.unescape(s)` — entity encoding

### xml.etree.ElementTree
- R4: `ET.parse(filename)` — parse XML file
- R5: `ET.fromstring(text)` — parse XML string
- R6: Element: `.tag`, `.text`, `.attrib`, `.tail`
- R7: `.find(path)`, `.findall(path)`, `.iter(tag)`
- R8: `ET.tostring(element)` — serialize back to XML
- R9: `ET.SubElement(parent, tag)` — create child element

## Priority

P3 — needed for web scraping and config file parsing.
