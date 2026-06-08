---
id: pulsar-markup-xslt
type: spec
title: "XSLT Transformer"
version: 1
spec_type: utility
created_at: 2026-01-30T07:20:00.000000+00:00
updated_at: 2026-01-30T07:20:00.000000+00:00
requirements:
  total: 2
  ids: [R1, R2]
---

<spec>

# XSLT Transformer

## Overview

Basic XSLT 1.0 transformation engine.

## Requirements

### R1 - Template Matching

```yaml
id: R1
priority: high
status: draft
```

Support XSLT elements:
- `<xsl:template match="...">`
- `<xsl:apply-templates>`
- `<xsl:value-of select="...">`
- `<xsl:for-each select="...">`
- `<xsl:if test="...">`
- `<xsl:choose>`, `<xsl:when>`, `<xsl:otherwise>`

### R2 - Output Control

```yaml
id: R2
priority: medium
status: draft
```

Support output methods:
- `<xsl:output method="xml|html|text">`
- `<xsl:copy>`, `<xsl:copy-of>`
- `<xsl:element>`, `<xsl:attribute>`

## Acceptance Criteria

### Scenario: Transform XML

- **GIVEN** XML document and XSLT stylesheet
- **WHEN** Call `transform(xml, xslt)`
- **THEN** Transformed output returned

</spec>
