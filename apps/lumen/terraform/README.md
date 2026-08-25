# Lumen installation Terraform

Terraform for the long-lived GCP substrate used by current Managed Lumen
installations. It has two independent modules.

```text
modules/lumen-pki/       private trust and certificate-request authority
modules/lumen-capacity/  shared GKE node pools and the in-cluster catalog
examples/installation/   one root that composes both against an existing cluster
```

The current operator reads `lumen-system/lumen-capacity-catalog` for legacy
Managed placement cases. A non-empty `nodeSelector` with the default machine
type uses a bounded Kubernetes-native compatibility path and does not need the
catalog. A plain
Kubernetes-native placement contract is a future outcome in
[Kubernetes-native placement](../ROADMAP.md#kubernetes-native-placement). The
catalog is a compatibility substrate. It is not the long-term public API.

## Ownership boundary

The PKI module owns a private CA hierarchy and the narrow authority to request
certificates from it. The capacity module owns shared GKE node pools and the
ConfigMap that publishes their supported machine types, selectors, limits, and
lifecycle state.

In the target GKE profile, the platform owns custom ComputeClasses, node
auto-provisioning, StorageClasses, and the issuer. New Lumen manifests select
those platform objects through standard Kubernetes fields. This Terraform
module does not make a GCE machine type part of the portable Lumen contract.

Neither module owns the GCP project, VPC, GKE cluster, DNS zone, target Lumen
namespaces, Lumen custom resources, certificates, private keys, bearer tokens,
or application data. These values are inputs or runtime objects.

No leaf certificate, private key, or bearer credential is a Terraform-managed
value. The in-cluster certificate controller authenticates to CA Service and
fetches its own material. The practical consequence is:

> Re-running `terraform apply` reconciles PKI *configuration*. It is never how a
> certificate gets renewed, and nothing here has to run when a Pod is replaced.

The one-line test for anything proposed for this tree is whether it would have
to run again when a certificate expires. If yes, it belongs in the controller.

The planned Lumen operator will own that leaf lifecycle. It will request and
rotate separate serving and peer certificates, write runtime Secrets, and
publish public serving trust. Terraform will continue to own only issuer and
permission substrate.

## Current capacity catalog

This section describes legacy compatibility that the current operator still
requires. It does not define the future Kubernetes-native placement API.

`modules/lumen-capacity` creates one shared GKE node pool per declared direct
GCE machine type. It publishes one ConfigMap through the Kubernetes provider.

Default identity:

| Field | Value |
|---|---|
| Namespace | `lumen-system` |
| ConfigMap | `lumen-capacity-catalog` |
| Data key | `catalog.json` |
| Selector key | `lumen.axiom.dev/capacity-profile` |

Each `capacity_profiles` entry needs a direct machine type and explicit
`max_nodes`. A legacy Managed `Lumen` also selects a direct machine type through
`spec.placement.initialMachineType`. The operator rejects a missing, malformed,
draining, full, or incompatible catalog on that path. A non-empty selector
with the default machine type is the bounded fallback to plain `nodeSelector`
and `tolerations`.

The module can retain a draining profile and requires an explicit retirement
acknowledgement before it removes a pool recorded as in use. Terraform owns the
catalog. Operators should repair its input and reapply instead of hand-editing
the ConfigMap.

## The trust boundary

One trust domain per environment, with regional issuing capacity — not one CA
per Lumen namespace. Splitting trust per namespace multiplies the number of
roots an operator has to keep track of without making any single compromise
smaller.

Trusting the CA is never authorization. A verifier that accepts any certificate
this pool signed has learned nothing useful; it must still check the exact
service DNS name or peer identity it expected. The pool's job is to make sure
the names it certifies are ones Lumen could legitimately have.

Issuance is constrained at the pool, where the CA enforces it, rather than at
the requester:

| Constraint | Value |
|---|---|
| Leaf lifetime ceiling | `max_leaf_lifetime_seconds`, default 24h |
| CA capability | none — `is_ca = false`, path length 0, no `cert_sign`/`crl_sign` |
| Usages | `serverAuth` + `clientAuth` (a Raft peer is both) |
| DNS SANs | must end in one of `allowed_dns_suffixes`, all cluster-internal |
| URI SANs | must be `spiffe://<trust_domain>/ns/…` |
| Subject | passthrough off; config-based issuance off |

The SAN rule is a CEL expression over `subject_alt_names.all(...)`. `all`, not
`exists`: with `exists`, a request carrying one valid cluster name alongside one
attacker-chosen public name satisfies the policy and gets *both* certified. That
substitution is a single word and completely silent, which is why
`tests/issuance_policy.tftest.hcl` asserts on the operator directly.

## Who may ask for a certificate

Exactly one named ServiceAccount, bound through Workload Identity as a direct
federated principal:

```
principal://iam.googleapis.com/projects/<number>/locations/global
  /workloadIdentityPools/<pool>/subject/ns/<namespace>/sa/<service-account>
```

No intermediate Google service account, and therefore no long-lived credential
to create, store, rotate, or leak. The grant is request + read only.
`additional_controller_roles` widens it from a three-role allowlist; CA admin,
project editor, service-account-key creation, Compute, and GKE roles are not
expressible through this module at all, so a role nobody anticipated is refused
by default rather than accepted by omission.

`certificate_controller.service_account` rejects `*`, `default`, and `system`.
A wildcard there would hand every workload in the namespace the right to mint a
Lumen-trusted identity.

## PKI modes

`ca_pool_mode = "create"` provisions a protected self-signed root plus an
in-region subordinate issuer. `ca_pool_mode = "existing"` references a pool an
operator already approved and creates **no** CA resources — the IAM binding is
the only thing it declares.

Once a created hierarchy is issuing (`created_hierarchy_in_use = true`),
switching to `existing` is refused. Orphaning a live root is a decision, not an
input edit, so it travels through `retirement.acknowledged` with a written
reason — and that same acknowledgement is what releases deletion protection on
the CAs. Nothing else does.

## Composing it

`examples/installation/` is the root: it configures the provider, reads the
existing cluster, and instantiates both modules. The modules share cluster
identity inputs, but neither reads an output from the other. A capacity edit
does not plan a PKI change. A PKI edit does not plan a node-pool change.

At least one active capacity profile must match every Managed instance's
machine type. An empty catalog can be valid Terraform output, but it cannot
place the current default `e2-standard-2` Lumen instance.

## Running the gate

```bash
bash acceptance/gcp/scripts/check.sh
```

That runs `fmt`, `validate`, and `terraform test` over both directories, plus
the ownership oracle in `acceptance/gcp/tests/lumen_pki_ownership.sh`.

The fixtures run against a mocked provider: **no GCP project, no credential,
nothing billable.** That is what makes the rejects — a public DNS suffix, a zone
where a region belongs, `roles/editor`, an issuer outliving its root, an
unacknowledged retirement — a per-commit gate instead of something only a cloud
run would notice. `expect_failures` makes Terraform's own validation engine the
oracle, so these are real refusals, not string matches.

The ownership oracle is an allowlist of the resource *types* each configuration
may declare. A denylist would only refuse what someone thought to forbid; it
would not catch a certificate resource under a name nobody predicted. Widening
the boundary means editing that expected set, in a diff whose entire content is
the boundary being moved.

Cross-variable validation puts the floor at Terraform 1.9.

## Not used here

GKE Managed Workload Identities — the Preview feature that projects fleet SVIDs
through a SPIFFE CSI driver — may replace the controller later behind the same
runtime contract. Nothing production-required depends on it today, and R9's gate
greps for its API group by name, so this paragraph names it in prose only: a
document explaining that we do not use something must not be what makes the
check for it fire.
