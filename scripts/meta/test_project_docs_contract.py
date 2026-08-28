#!/usr/bin/env python3
"""Tests for the project product-document set validator."""

from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("project_docs_contract.py")
SPEC = importlib.util.spec_from_file_location("project_docs_contract", SCRIPT)
assert SPEC and SPEC.loader
project_docs_contract = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = project_docs_contract
SPEC.loader.exec_module(project_docs_contract)


README = """# Demo

## Brief

Demo indexes caller-owned values and returns caller-owned IDs.

## Primary workflow

1. Declare a schema.
2. Index a value.
3. Query the index.

## Public operations

Use `PUT /items` and `POST /items/search`.

## Contract discovery

Run `demo spec` for the wire contract.

## Capabilities

Every entry is a product capability.

### Capability index

| Capability | ID | User promise | Sources |
|---|---|---|---|
| Search | `search` | Return matching caller IDs. | `apps/demo`, `libs/shared`, `external:runtime` |

### Search

- ID: `search`
- Promise: Return matching caller IDs.
- Sources:
  - [`apps/demo`](./) defines query behavior and product composition.
  - [`libs/shared`](../../libs/shared/README.md) provides reusable index mechanics.
  - `external:runtime` runs the declared service contract.
- Gate: `scripts/check-demo.sh`

## Supporting documents

- [Status](STATUS.md)
- [Roadmap](ROADMAP.md)
- [Contributing](CONTRIBUTING.md)
"""

STATUS = """# Demo status

## Scope

This document reports the current source contract.

## State definitions

| State | Meaning |
|---|---|
| Supported | A contract, implementation, and executable gate exist. |
| Limited | The stated scope works within an explicit material boundary. |
| Not supported | The current product contract does not include this behavior. |

## Support matrix

| Surface | ID | State | Supported scope | Limits | Evidence |
|---|---|---|---|---|---|
| Search | `search` | Supported | Return matching caller IDs. | Source records remain outside Demo. | `scripts/check-demo.sh` |
| Automatic scaling | `automatic-scaling` | Not supported | No controller changes the replica count. | The design needs membership changes. | [Safe autoscaling](ROADMAP.md#safe-autoscaling) |

## Evidence policy

Commands are required gates. This file does not record a test run.
"""

ROADMAP = """# Demo roadmap

## Purpose

This document records future outcomes and non-goals.

## Near-term outcomes

### Safe autoscaling

- ID: `safe-autoscaling`
- Outcome: Demo adds or removes replica layers safely.
- Boundary: Membership changes happen before workload size changes.
- Completion evidence: A failure test proves quorum and acknowledged writes survive.
- Tracking: Not assigned.

## Later outcomes

No items.

## Non-goals

### Source record ownership

- ID: `source-record-ownership`
- Reason: Callers keep and hydrate their own source records.
"""

PROTOCOL = """# Demo protocol

## Purpose

This guide maps protocol facts to their maintained sources.

## Contract map

| Fact | Canonical source | Discovery |
|---|---|---|
| HTTP shapes | Demo OpenAPI | `demo spec` |
| Request behavior | Demo workflow topic | `demo llm --topic workflow` |

## Use the protocol

Create an item, index its values, and query the resulting IDs.

## Current boundaries

See the [support matrix](../STATUS.md#support-matrix) for current limits.

## Supporting documents

- [Product README](../README.md)
- [Roadmap](../ROADMAP.md)
"""

CLIENTS = """# Demo generated clients

## Contract

Demo generates source from its OpenAPI document. It does not publish packages.

## Generate

Run `demo spec gen --lang ts --out generated`.

## Language matrix

| Language | Generated form | Transport | Auth input | Current limits |
|---|---|---|---|---|
| TypeScript | Async client | HTTP/1.1 | Fixed headers | No streaming method |

## Connect

Set the service base URL before the first request.

## Current boundaries

See the [support matrix](../STATUS.md#support-matrix) for current limits.

## Verification

Run `scripts/check-demo.sh`.

## Supporting documents

- [Product README](../README.md)
- [Roadmap](../ROADMAP.md)
"""

INDEXING = """# Demo indexing

## Purpose

This guide separates current indexing behavior from future write outcomes.

## Contract map

| Fact | Canonical source | Discovery |
|---|---|---|
| Current wire shape | Demo OpenAPI | `demo spec` |
| Current support | [STATUS](../STATUS.md) | Read the support matrix. |

## Data ownership

The caller owns source records. Demo owns only the derived index.

## Schema contract

The current schema supports the field types declared by `demo spec`.

## Write contract

Current writes merge caller-owned values. Future writes remain in the roadmap.

## Durability

The selected runtime backend defines the current durability boundary.

## Rebuild and activation

The current rebuild writes the active index. Future generations are not supported.

## Current boundaries

See the [support matrix](../STATUS.md#support-matrix) for current limits.

## Supporting documents

- [Product README](../README.md)
- [Roadmap](../ROADMAP.md)
"""

QUERYING = """# Demo querying

## Purpose

This guide separates current queries from the future query contract.

## Data ownership

Demo selects caller IDs. The caller hydrates source records.

## Contract map

| Fact | Canonical source | Discovery |
|---|---|---|
| Current wire shape | Demo OpenAPI | `demo spec` |
| Current support | [STATUS](../STATUS.md) | Read the support matrix. |

## Search model

The current query model is declared by the OpenAPI document.

## Result controls

The current response returns caller IDs and result metadata.

## Facets and metrics

Facets remain a future outcome in the [roadmap](../ROADMAP.md#safe-autoscaling).

## Limits and failures

The current runtime enforces the limits in the support matrix.

## Compatibility and migration

Use the linked migration guide for version-specific changes.

## Current boundaries

See the [support matrix](../STATUS.md#support-matrix) for current limits.

## Supporting documents

- [Product README](../README.md)
- [Roadmap](../ROADMAP.md)
"""

GKE = """# Demo GKE

## Purpose

This guide defines the current and target GKE deployment contract.

## Standalone GKE instance

Standalone renders one StatefulSet and one separately owned PVC instance.

## Contract map

| Fact | Canonical source | Discovery |
|---|---|---|
| Current support | [STATUS](../STATUS.md) | Read the support matrix. |
| Future outcomes | [ROADMAP](../ROADMAP.md) | Read the named outcomes. |

## Support tiers

Standalone is the local path. Managed GKE support is stated in STATUS.

## Runtime size and topology

One pod is not HA. Replicated topology is a separate choice from Fleet.

## Kubernetes-native contract

The portable API uses Kubernetes resources, storage, and placement inputs.

## GKE Standard Regional profile

The target profile uses a regional Standard cluster.

## Storage, placement, and disruption

Storage topology and application rollout have separate safety contracts.

## Identity and networking

Kubernetes ServiceAccount identity is separate from cloud workload identity.

## Verification

Run the project document checker and the named GKE acceptance gate.

## Current boundaries

The current repository does not prove a regional production profile.

## Supporting documents

- [Product README](../README.md)
- [Roadmap](../ROADMAP.md)
"""

CLIENT_INTEGRATION = """# Demo client integration

## Purpose

This guide assigns generated-client, workload-template, and caller work.

## Contract map

| Fact | Canonical source | Discovery |
|---|---|---|
| Current clients | [Client guide](../clients/README.md) | Read the language matrix. |
| Current support | [STATUS](../STATUS.md) | Read the support matrix. |

## Responsibility boundary

The server, generated client, workload manifest, and caller own separate work.

## Connection profiles

Standalone and Managed connections use explicit profiles.

## Generated client behavior

Generated clients keep current and future behavior separate.

## Kubernetes workload template

A future template projects identity without storing a token Secret.

## Source integration

The caller fetches source records and restores search result order.

## Failure handling

Retry behavior depends on operation safety and idempotency.

## Verification

Run the project document checker and generated-client gates.

## Current boundaries

The future workload template is not present now.

## Supporting documents

- [Product README](../README.md)
- [Generated clients](../clients/README.md)
- [Roadmap](../ROADMAP.md)
"""

MIGRATION = """# Demo 0.5 search migration

## Purpose

This guide moves callers from the 0.4 search contract to the 0.5 contract.

## Compatibility window

| Surface | 0.4.x | 0.5.0 | Required action |
|---|---|---|---|
| Total count | JSON number | Typed count object | Update response decoding. |

## Schema migration

Rebuild fields whose declared type changes.

## Request migration

Translate supported request fields before the compatibility window closes.

## Response migration

Accept the new typed count response before upgrading the server.

## Managed activation

Finalize compatibility only after every serving member supports the new contract.

## Migration tools

Use the offline migration command. It does not contact a server.

## Verification

Run the project document checker and the migration fixtures.

## Supporting documents

- [Product README](../README.md)
- [Current support](../STATUS.md)
"""


class ProjectDocsContractTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.repo = Path(self.temp.name)
        self.project = self.repo / "apps/demo"
        self.project.mkdir(parents=True)
        (self.repo / "libs/shared").mkdir(parents=True)
        (self.repo / "scripts").mkdir()
        (self.project / "README.md").write_text(README, encoding="utf-8")
        (self.project / "STATUS.md").write_text(STATUS, encoding="utf-8")
        (self.project / "ROADMAP.md").write_text(ROADMAP, encoding="utf-8")
        (self.project / "CONTRIBUTING.md").write_text("# Demo contributing\n", encoding="utf-8")
        (self.repo / "libs/shared/README.md").write_text("# Shared\n", encoding="utf-8")
        (self.repo / "scripts/check-demo.sh").write_text("#!/bin/sh\n", encoding="utf-8")

    def tearDown(self) -> None:
        self.temp.cleanup()

    def validate(self):
        return project_docs_contract.validate_project(self.repo, self.project, {})

    def adopt_supporting_docs(self) -> None:
        protocol = self.project / "docs/protocol.md"
        clients = self.project / "clients/README.md"
        protocol.parent.mkdir()
        clients.parent.mkdir()
        protocol.write_text(PROTOCOL, encoding="utf-8")
        clients.write_text(CLIENTS, encoding="utf-8")
        readme = README.replace(
            "- [Contributing](CONTRIBUTING.md)\n",
            "- [Protocol](docs/protocol.md)\n"
            "- [Generated clients](clients/README.md)\n"
            "- [Contributing](CONTRIBUTING.md)\n",
        )
        (self.project / "README.md").write_text(readme, encoding="utf-8")

    def adopt_extended_supporting_docs(self) -> None:
        self.adopt_supporting_docs()
        (self.project / "docs/indexing.md").write_text(INDEXING, encoding="utf-8")
        (self.project / "docs/querying.md").write_text(QUERYING, encoding="utf-8")
        (self.project / "docs/migration-0.5-search.md").write_text(
            MIGRATION, encoding="utf-8"
        )
        readme_path = self.project / "README.md"
        readme = readme_path.read_text(encoding="utf-8").replace(
            "- [Contributing](CONTRIBUTING.md)\n",
            "- [Indexing](docs/indexing.md)\n"
            "- [Querying](docs/querying.md)\n"
            "- [Migration](docs/migration-0.5-search.md)\n"
            "- [Contributing](CONTRIBUTING.md)\n",
        )
        readme_path.write_text(readme, encoding="utf-8")

    def adopt_environment_and_client_integration_docs(self) -> None:
        self.adopt_extended_supporting_docs()
        (self.project / "docs/gke.md").write_text(GKE, encoding="utf-8")
        (self.project / "docs/client-integration.md").write_text(
            CLIENT_INTEGRATION, encoding="utf-8"
        )
        readme_path = self.project / "README.md"
        readme = readme_path.read_text(encoding="utf-8").replace(
            "- [Contributing](CONTRIBUTING.md)\n",
            "- [GKE](docs/gke.md)\n"
            "- [Client integration](docs/client-integration.md)\n"
            "- [Contributing](CONTRIBUTING.md)\n",
        )
        readme_path.write_text(readme, encoding="utf-8")

    @staticmethod
    def all_rules(report) -> set[str]:
        rules = {finding.rule for finding in report.findings}
        if report.readme:
            rules.update(finding.rule for finding in report.readme.findings)
        if report.status:
            rules.update(finding.rule for finding in report.status.findings)
        if report.roadmap:
            rules.update(finding.rule for finding in report.roadmap.findings)
        for document in report.supporting_docs:
            rules.update(finding.rule for finding in document.findings)
        return rules

    def test_valid_document_set_passes(self) -> None:
        report = self.validate()
        self.assertTrue(report.ok, report.as_dict())
        self.assertEqual(len(report.status.surfaces), 2)
        self.assertEqual(len(report.roadmap.items), 2)
        self.assertEqual(report.supporting_docs, [])

    def test_all_three_documents_are_required(self) -> None:
        (self.project / "STATUS.md").unlink()
        report = self.validate()
        self.assertIn("P1", self.all_rules(report))

    def test_readme_must_link_both_companion_documents(self) -> None:
        path = self.project / "README.md"
        path.write_text(README.replace("- [Roadmap](ROADMAP.md)\n", ""), encoding="utf-8")
        report = self.validate()
        self.assertIn("P2", self.all_rules(report))

    def test_readme_link_must_target_the_companion_file(self) -> None:
        docs = self.project / "docs"
        docs.mkdir()
        (docs / "ROADMAP.md").write_text("# Other roadmap\n", encoding="utf-8")
        path = self.project / "README.md"
        path.write_text(README.replace("(ROADMAP.md)", "(docs/ROADMAP.md)"), encoding="utf-8")
        report = self.validate()
        self.assertIn("P2", self.all_rules(report))

    def test_companion_titles_follow_the_readme_title(self) -> None:
        path = self.project / "STATUS.md"
        path.write_text(STATUS.replace("# Demo status", "# Other status"), encoding="utf-8")
        report = self.validate()
        self.assertIn("P5", self.all_rules(report))

    def test_supported_surface_requires_resolvable_gate(self) -> None:
        path = self.project / "STATUS.md"
        path.write_text(STATUS.replace("`scripts/check-demo.sh`", "No gate exists."), encoding="utf-8")
        report = self.validate()
        self.assertIn("S5", self.all_rules(report))

    def test_not_supported_surface_requires_known_roadmap_link(self) -> None:
        path = self.project / "STATUS.md"
        path.write_text(
            STATUS.replace("ROADMAP.md#safe-autoscaling", "ROADMAP.md#missing-outcome"),
            encoding="utf-8",
        )
        report = self.validate()
        rules = self.all_rules(report)
        self.assertIn("S7", rules)
        self.assertIn("P3", rules)

    def test_limited_surface_requires_roadmap_destination(self) -> None:
        path = self.project / "STATUS.md"
        path.write_text(
            STATUS.replace("| Search | `search` | Supported", "| Search | `search` | Limited"),
            encoding="utf-8",
        )
        report = self.validate()
        self.assertIn("S6", self.all_rules(report))

    def test_roadmap_rejects_progress_and_unstable_heading_id(self) -> None:
        path = self.project / "ROADMAP.md"
        text = ROADMAP.replace(
            "- Outcome: Demo adds or removes replica layers safely.",
            "- Status: active\n- Outcome: Demo adds or removes replica layers safely.",
        ).replace("`safe-autoscaling`", "`different-id`")
        path.write_text(text, encoding="utf-8")
        report = self.validate()
        rules = self.all_rules(report)
        self.assertIn("M2", rules)
        self.assertIn("M4", rules)

    def test_current_surface_cannot_also_be_future_work(self) -> None:
        path = self.project / "STATUS.md"
        path.write_text(
            STATUS.replace("| Search | `search` | Supported", "| Search | `safe-autoscaling` | Supported"),
            encoding="utf-8",
        )
        report = self.validate()
        self.assertIn("P4", self.all_rules(report))

    def test_limited_surface_can_share_its_completion_outcome_id(self) -> None:
        path = self.project / "STATUS.md"
        path.write_text(
            STATUS.replace(
                "| Search | `search` | Supported | Return matching caller IDs. | Source records remain outside Demo. |",
                "| Search | `safe-autoscaling` | Limited | Return matching caller IDs. | Source records remain outside Demo. See [Safe autoscaling](ROADMAP.md#safe-autoscaling). |",
            ),
            encoding="utf-8",
        )
        report = self.validate()
        self.assertNotIn("P4", self.all_rules(report))

    def test_prompt_binds_all_three_files_without_leaking_answers(self) -> None:
        report = self.validate()
        self.assertTrue(report.ok, report.as_dict())
        prompt = project_docs_contract.clean_reader_prompt(report, self.project)
        self.assertIn(report.readme.sha256, prompt)
        self.assertIn(report.status.sha256, prompt)
        self.assertIn(report.roadmap.sha256, prompt)
        self.assertIn("Return JSON only", prompt)
        self.assertIn('"scope": "<supported scope>"', prompt)
        self.assertIn("integer from 0 to 100", prompt)
        self.assertNotIn("Safe autoscaling", prompt)

    def test_existing_supporting_document_must_be_linked(self) -> None:
        protocol = self.project / "docs/protocol.md"
        protocol.parent.mkdir()
        protocol.write_text(PROTOCOL, encoding="utf-8")
        report = self.validate()
        self.assertIn("P6", self.all_rules(report))

    def test_existing_indexing_or_querying_guide_must_be_linked(self) -> None:
        docs = self.project / "docs"
        docs.mkdir()
        (docs / "indexing.md").write_text(INDEXING, encoding="utf-8")
        (docs / "querying.md").write_text(QUERYING, encoding="utf-8")
        report = self.validate()
        self.assertIn("P6", self.all_rules(report))

    def test_existing_gke_or_client_integration_guide_must_be_linked(self) -> None:
        docs = self.project / "docs"
        docs.mkdir()
        (docs / "gke.md").write_text(GKE, encoding="utf-8")
        (docs / "client-integration.md").write_text(
            CLIENT_INTEGRATION, encoding="utf-8"
        )
        report = self.validate()
        self.assertIn("P6", self.all_rules(report))

    def test_unlinked_historical_migration_is_not_adopted(self) -> None:
        docs = self.project / "docs"
        docs.mkdir()
        (docs / "migration-old-change.md").write_text("historical\n", encoding="utf-8")
        report = self.validate()
        self.assertTrue(report.ok, report.as_dict())
        self.assertEqual(report.supporting_docs, [])

    def test_linked_supporting_document_must_exist(self) -> None:
        readme = README.replace(
            "- [Contributing](CONTRIBUTING.md)\n",
            "- [Protocol](docs/protocol.md)\n- [Contributing](CONTRIBUTING.md)\n",
        )
        (self.project / "README.md").write_text(readme, encoding="utf-8")
        report = self.validate()
        self.assertIn("P6", self.all_rules(report))

    def test_adopted_supporting_documents_pass_and_appear_in_json(self) -> None:
        self.adopt_supporting_docs()
        report = self.validate()
        self.assertTrue(report.ok, report.as_dict())
        self.assertEqual(
            [document.kind for document in report.supporting_docs],
            ["protocol", "clients"],
        )
        encoded = report.as_dict()["supporting_docs"]
        self.assertEqual([document["kind"] for document in encoded], ["protocol", "clients"])

    def test_extended_supporting_documents_pass_and_appear_in_json(self) -> None:
        self.adopt_extended_supporting_docs()
        report = self.validate()
        self.assertTrue(report.ok, report.as_dict())
        kinds = [document.kind for document in report.supporting_docs]
        self.assertEqual(kinds, ["protocol", "clients", "indexing", "querying", "migration"])

    def test_environment_and_client_integration_docs_pass_and_appear_in_json(self) -> None:
        self.adopt_environment_and_client_integration_docs()
        report = self.validate()
        self.assertTrue(report.ok, report.as_dict())
        kinds = [document.kind for document in report.supporting_docs]
        self.assertEqual(
            kinds,
            [
                "protocol",
                "clients",
                "indexing",
                "querying",
                "gke",
                "client-integration",
                "migration",
            ],
        )

    def test_supporting_document_requires_fixed_headings_and_table(self) -> None:
        self.adopt_supporting_docs()
        protocol = self.project / "docs/protocol.md"
        protocol.write_text(
            PROTOCOL.replace("## Use the protocol", "## Protocol usage").replace(
                "| Fact | Canonical source | Discovery |",
                "| Fact | Source | Discovery |",
            ),
            encoding="utf-8",
        )
        report = self.validate()
        rules = self.all_rules(report)
        self.assertIn("D1", rules)
        self.assertIn("D3", rules)

    def test_querying_and_migration_require_fixed_headings_and_tables(self) -> None:
        self.adopt_extended_supporting_docs()
        querying = self.project / "docs/querying.md"
        querying.write_text(
            QUERYING.replace("## Search model", "## Query model").replace(
                "| Fact | Canonical source | Discovery |",
                "| Fact | Source | Discovery |",
            ),
            encoding="utf-8",
        )
        migration = self.project / "docs/migration-0.5-search.md"
        migration.write_text(
            MIGRATION.replace("| Surface | 0.4.x | 0.5.0 | Required action |", "| Surface | Before | After | Action |"),
            encoding="utf-8",
        )
        report = self.validate()
        rules = self.all_rules(report)
        self.assertIn("D1", rules)
        self.assertIn("D3", rules)

    def test_gke_and_client_integration_require_fixed_headings_and_tables(self) -> None:
        self.adopt_environment_and_client_integration_docs()
        gke = self.project / "docs/gke.md"
        gke.write_text(
            GKE.replace("## Standalone GKE instance\n\n", "")
            .replace("## Support tiers", "## Environments")
            .replace(
                "| Fact | Canonical source | Discovery |",
                "| Fact | Source | Discovery |",
            ),
            encoding="utf-8",
        )
        client = self.project / "docs/client-integration.md"
        client.write_text(
            CLIENT_INTEGRATION.replace(
                "## Connection profiles", "## Connections"
            ),
            encoding="utf-8",
        )
        report = self.validate()
        rules = self.all_rules(report)
        self.assertIn("D1", rules)
        self.assertIn("D3", rules)

    def test_supporting_document_rejects_broken_link_and_anchor(self) -> None:
        self.adopt_supporting_docs()
        clients = self.project / "clients/README.md"
        clients.write_text(
            CLIENTS.replace("STATUS.md#support-matrix", "STATUS.md#missing-surface"),
            encoding="utf-8",
        )
        report = self.validate()
        self.assertIn("D4", self.all_rules(report))

    def test_supporting_document_path_resolves_to_project(self) -> None:
        self.adopt_supporting_docs()
        self.assertEqual(
            project_docs_contract.resolve_project(
                self.repo, str(self.project / "docs/protocol.md")
            ),
            self.project.resolve(),
        )

    def test_extended_supporting_document_paths_resolve_to_project(self) -> None:
        self.adopt_extended_supporting_docs()
        for relative in (
            "docs/indexing.md",
            "docs/querying.md",
            "docs/gke.md",
            "docs/client-integration.md",
            "docs/migration-0.5-search.md",
        ):
            if relative in {"docs/gke.md", "docs/client-integration.md"}:
                (self.project / relative).write_text(
                    GKE if relative == "docs/gke.md" else CLIENT_INTEGRATION,
                    encoding="utf-8",
                )
            self.assertEqual(
                project_docs_contract.resolve_project(
                    self.repo, str(self.project / relative)
                ),
                self.project.resolve(),
            )
        self.assertEqual(
            project_docs_contract.resolve_project(
                self.repo, str(self.project / "clients/README.md")
            ),
            self.project.resolve(),
        )

    def test_prompt_binds_supporting_documents_and_requests_interfaces(self) -> None:
        self.adopt_supporting_docs()
        report = self.validate()
        self.assertTrue(report.ok, report.as_dict())
        prompt = project_docs_contract.clean_reader_prompt(report, self.project)
        for document in report.supporting_docs:
            self.assertIn(document.sha256, prompt)
            self.assertIn(document.path.removeprefix("apps/demo/"), prompt)
        self.assertIn('"interfaces"', prompt)
        self.assertIn('"contract_map"', prompt)
        self.assertIn('"languages"', prompt)
        self.assertIn("Read only these 5 files", prompt)

    def test_prompt_binds_extended_guides_and_requests_contract_details(self) -> None:
        self.adopt_extended_supporting_docs()
        report = self.validate()
        self.assertTrue(report.ok, report.as_dict())
        prompt = project_docs_contract.clean_reader_prompt(report, self.project)
        for document in report.supporting_docs:
            self.assertIn(document.sha256, prompt)
        self.assertIn('"indexing"', prompt)
        self.assertIn('"querying"', prompt)
        self.assertIn('"migrations"', prompt)
        self.assertIn('"compatibility_rows"', prompt)
        self.assertIn("Read only these 8 files", prompt)

    def test_prompt_binds_gke_and_client_integration_contracts(self) -> None:
        self.adopt_environment_and_client_integration_docs()
        report = self.validate()
        self.assertTrue(report.ok, report.as_dict())
        prompt = project_docs_contract.clean_reader_prompt(report, self.project)
        self.assertIn('"gke"', prompt)
        self.assertIn('"client_integration"', prompt)
        self.assertIn('"runtime_topologies"', prompt)
        self.assertIn('"workload_template"', prompt)
        self.assertIn("Read only these 10 files", prompt)

    def test_lumen_specific_reversed_contract_assertions_are_refused(self) -> None:
        lumen = self.repo / "apps/lumen"
        lumen.mkdir(parents=True)
        bad = lumen / "README.md"
        reversed_claims = (
            "Fleet is HA.",
            "Fleet provides autoscaling.",
            "1 Pod is HA.",
            "2 voters are production HA.",
            "Zonal acceptance proves regional HA.",
            "PDB limits StatefulSet rolling updates.",
            "GCE machine types belong in the Kubernetes-native CRD.",
            "Operator creates the Kubernetes cluster.",
            "Operator creates namespace.",
            "Operator creates KSA.",
            "Operator creates client Deployment.",
            "Binding a KSA automatically adds an Authorization header.",
            "Generated clients support token rotation and safe retry today.",
            "Client-side validation replaces TokenReview.",
            "The current runtime Kustomize template is Managed.",
            "Lumen executes embedding models.",
            "Generated SDK packages are published.",
        )
        for claim in reversed_claims:
            with self.subTest(claim=claim):
                bad.write_text(f"# Lumen\n\n{claim}\n", encoding="utf-8")
                findings = project_docs_contract.validate_lumen_assertions(
                    lumen, [bad]
                )
                self.assertEqual([finding.rule for finding in findings], ["P7"])

    def test_lumen_specific_rules_allow_explicit_current_boundaries(self) -> None:
        lumen = self.repo / "apps/lumen"
        lumen.mkdir(parents=True)
        good = lumen / "README.md"
        good.write_text(
            "\n".join(
                (
                    "# Lumen",
                    "",
                    "Fleet is not HA or autoscaling.",
                    "One Pod is not HA.",
                    "A PDB does not limit a StatefulSet rolling update.",
                    "Generated clients do not support token rotation today.",
                    "Lumen does not execute an embedding model.",
                )
            ),
            encoding="utf-8",
        )
        findings = project_docs_contract.validate_lumen_assertions(lumen, [good])
        self.assertEqual(findings, [])

    def test_non_lumen_projects_do_not_receive_lumen_semantic_rules(self) -> None:
        bad = self.project / "BAD.md"
        bad.write_text("Fleet is HA.\n", encoding="utf-8")
        findings = project_docs_contract.validate_lumen_assertions(
            self.project, [bad]
        )
        self.assertEqual(findings, [])


if __name__ == "__main__":
    unittest.main()
