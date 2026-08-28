# Authenticate to Lumen

## Contract state

This guide separates current Standalone and Managed behavior from the planned
Managed access contract.

- Sections marked **Current** describe the current repository source.
- Sections marked **Planned** describe the target contract. They are not
  implemented yet.

The planned `access` fields, whole-runtime `lumenruntimes/use` permission,
`AccessPolicyReady` and `ClientTrustReady` conditions, operator-managed leaf
certificates, public CA ConfigMaps, and explicit ManagedKsa client profile do
not exist in the current source. The Standalone default-token provider does.

## Choose the runtime mode

### Standalone

**Current.** `lumen serve` defaults to `LUMEN_AUTH=off` and listens on
`127.0.0.1`. Compose keeps that auth-off behavior on a loopback port.

Standalone GKE uses `LUMEN_AUTH=in-cluster`. TokenReview authenticates the
calling KSA from the mounted token for the calling Pod's configured KSA.
The server checks that KSA against `allowedServiceAccounts`. SubjectAccessReview
and RBAC authorize every protected operation. The renderer creates one private
ClusterIP,
NetworkPolicy, the allowed application RoleBindings, and a separate management
ServiceAccount.

ClusterIP and NetworkPolicy control reachability only. TokenReview authenticates
the calling KSA; SubjectAccessReview and RBAC are the authorization boundary.
Never derive the NetworkPolicy namespace list from the KSA allow-list.

For an SDK client running at an exact `.svc.cluster.local` URL, the generated
client uses the mounted token for the calling Pod's configured KSA automatically.
This does not add a client configuration knob.

The token file is reread for each request. Automatic KSA use is restricted to
exact `.svc.cluster.local` URLs. Local, browser, and external clients do not
read or send a KSA token. An explicit `Authorization` header always wins.

The `lumen@0.4.29` release gate is a controller-run manual live GKE check. It
must verify both this Standalone `in-cluster` path and the inherited Managed
`required` path. The required continuity checks use a projected
`lumen.axiom.dev` token for success, the same KSA's default token for `401`,
and a projected unlisted KSA for `403`. The gate also verifies cleanup and a
sanitized receipt. The gate is not claimed as passed here until the controller
records that receipt. Cluster credentials, private inputs, and exact execution
stay with the controller.

### Managed

**Current.** A Managed `Lumen` resource defaults to `auth: required`. The
runtime validates a Kubernetes ServiceAccount token through TokenReview and
checks each operation through SubjectAccessReview. The CRD still accepts
`auth: disabled`.

Current access is external to Fleet. The `lumen k8s access render` command can
render a client ServiceAccount, TokenRequest issuer RBAC, and per-collection or
instance-admin RBAC. An operator or platform owner applies that bundle.

**Planned.** Managed requires Kubernetes ServiceAccount authentication. During
the planned 0.4.x migration, `auth: disabled` remains accepted but emits a
deprecation. Version 0.5.0 rejects it.

## Keep the identities separate

Managed operation uses three Kubernetes ServiceAccounts for different jobs:

| Identity | Purpose | Credential use |
|---|---|---|
| Operator ServiceAccount | Watches CRDs and reconciles Kubernetes objects. | Calls the Kubernetes API. It does not become the caller of normal search requests. |
| Runtime ServiceAccount | Runs a Lumen data-plane pod. | Calls TokenReview and SubjectAccessReview. Internal control-plane calls can use a separate Lumen-audience projection. |
| Client ServiceAccount | Identifies one application workload that calls Lumen. | Presents a short-lived token with audience `lumen.axiom.dev` to the runtime. |

A human developer keeps their kubeconfig identity. They do not send that human
identity directly to Lumen. They use TokenRequest to mint a short-lived token
for an allowed client ServiceAccount.

Binding a Deployment to a ServiceAccount does not add a Lumen token to an HTTP
request. The workload template must project the Lumen-audience token. The
client must read it and add the Authorization header.

A KSA token authenticates a workload to Lumen. Workload Identity Federation
lets an operator, backup job, or CA integration call a Google API. These are
different credentials and different audiences. Neither replaces serving TLS or
peer TLS.

## Current Managed request flow

**Current.** One protected request follows this path:

1. The caller obtains an audience-bound ServiceAccount token.
2. The caller sends `Authorization: Bearer <token>` over the serving TLS
   connection.
3. Lumen asks TokenReview for audience `lumen.axiom.dev`.
4. Lumen accepts only a username shaped as
   `system:serviceaccount:<namespace>:<name>`.
5. Lumen maps the operation to current per-collection or instance-admin
   resource attributes.
6. Lumen asks SubjectAccessReview in the runtime namespace.
7. Lumen serves the request only when Kubernetes returns an explicit allow.

Current authorization uses these virtual resources:

| Operation | API group | Resource | Resource name | Verb |
|---|---|---|---|---|
| Read one collection | `lumen.axiom.dev` | `lumencollections` | Collection ID | `get` |
| Write one collection | `lumen.axiom.dev` | `lumencollections` | Collection ID | `update` |
| Administer one collection | `lumen.axiom.dev` | `lumencollections` | Collection ID | `delete` |
| Use an instance-admin API | `lumen.axiom.dev` | `lumenadmin` | None | Operation role verb |

Fleet does not create these access Roles or RoleBindings. Managed clients still
need an explicit audience-bound Authorization value. The automatic default
ServiceAccount token path belongs only to Standalone in-cluster URLs.

## Current Standalone in-cluster request flow

**Current.** One protected Standalone request follows this path:

1. The generated client targets an exact non-empty `*.svc.cluster.local` HTTP
   or HTTPS URL.
2. It rereads `/var/run/secrets/kubernetes.io/serviceaccount/token` immediately
   before the request and sends it as `Authorization: Bearer <token>`.
3. Lumen asks TokenReview to use the API server's default audiences.
4. Lumen accepts only a username shaped as
   `system:serviceaccount:<namespace>:<name>`.
5. Lumen maps the operation to `lumencollections` or `lumenadmin`.
6. Lumen asks SubjectAccessReview in the Standalone namespace.

An allowed application KSA receives collection, schema, index, and search
access. A missing or bad token returns `401`. A TokenReview-valid but unlisted
KSA returns `403`. A listed KSA receives collection, schema, index, and search
access, but `/admin/*` returns `403`; only `<name>-admin` receives admin RBAC.
The renderer creates `<name>-admin` for backup and restore. The runtime Service
ClusterIP and NetworkPolicy limit reachability to cluster workloads.

## Planned whole-runtime access

**Planned.** Direct Managed and Fleet use one typed access shape:

```yaml
access:
  allowedServiceAccounts:
    - namespace: app-a
      name: search-client
```

Direct Managed stores it at `Lumen.spec.access`.

Fleet stores a common list at `LumenFleet.spec.defaults.access`. An instance
can replace that complete list at `LumenFleet.spec.instances[].access`:

```yaml
apiVersion: lumen.dev/v1alpha1
kind: LumenFleet
metadata:
  name: search
spec:
  defaults:
    access:
      allowedServiceAccounts:
        - namespace: app-a
          name: search-client
  instances:
    - namespace: search-a
    - namespace: search-b
      access:
        allowedServiceAccounts:
          - namespace: app-b
            name: search-client
```

The `search-b` list replaces the defaults. It does not form a union with them.
An explicit empty list is valid and means deny all:

```yaml
access:
  allowedServiceAccounts: []
```

Every entry must contain one exact namespace and ServiceAccount name. Wildcard
names, ServiceAccount namespace groups, and raw Kubernetes usernames are
invalid.

`instances[].spec` remains an advanced RFC 7386 patch for other `LumenSpec`
fields. It must not set `access`. An entry that tries to set
`instances[].spec.access` is rejected so access never has two sources.

Fleet does not create a ServiceAccount, namespace, client Deployment, token
Secret, or Kubernetes cluster.

## Planned RBAC and readiness

**Planned.** The operator creates one namespaced Role and the required
RoleBindings for each runtime. The subject of a RoleBinding can name a client
ServiceAccount from another namespace.

The runtime maps every protected API to one whole-runtime SubjectAccessReview:

| Attribute | Value |
|---|---|
| API group | `lumen.axiom.dev` |
| Resource | `lumenruntimes` |
| Resource name | Child `Lumen` name |
| Verb | `use` |
| Namespace | Child `Lumen` namespace |

`use` permits all protected search, indexing, collection, and administration
APIs for that runtime. Per-collection grants are not part of the target model.
One runtime is therefore one complete trust boundary. Give `use` only to a KSA
that can be trusted with query, index, collection-management, and admin work.
This single permission is not a fine-grained least-privilege model. Use a
separate runtime when two clients need different permission levels.

Facet values are planned search metadata. When a field is declared
`facetable=true`, every KSA with this whole-runtime `use` permission can request
that field's facet values. Lumen does not add a field-level access control
list. A schema owner must treat `facetable` as an access disclosure decision.
Facets and whole-runtime access are both future work. See the
[querying access contract](querying.md#facet-value-access).

The operator publishes `AccessPolicyReady`:

- It is `False` when access Role or RoleBinding apply, adoption, or prune
  fails.
- `AccessPolicyReady=False` prevents the child `Ready` condition from becoming
  `True`.
- A successfully converged empty allow-list is deny-all and still reports
  `AccessPolicyReady=True`.

Kubernetes RBAC is the enforcement authority. A cluster administrator can
create another valid RoleBinding that grants `lumenruntimes/use`. Fleet cannot
prevent or revoke grants that another authorized owner creates outside its
ownership contract.

The planned operator also owns the runtime's leaf-certificate lifecycle and
public client trust. The platform supplies the issuer, CA policy, and any
Workload Identity Federation permission needed to call a Google API. The
operator keeps serving and peer certificates separate and stores their private
keys only in runtime Secrets.

For every allowed KSA namespace, the operator publishes one runtime-specific
public CA ConfigMap. Multiple allowed KSAs in that namespace share it. During a
root rotation, the bundle contains the old and new roots. The operator prunes
only ConfigMaps that it owns. An adoption or prune conflict sets
`ClientTrustReady=False` and prevents complete Managed readiness. The first
version does not use ClusterTrustBundle projection.

## Generated-client token flow

Generated clients keep the deployment profiles separate:

| Profile | Required inputs | Credential behavior |
|---|---|---|
| Local or Compose Standalone | Loopback or non-cluster URL | Never inspect a token or CA file. |
| In-cluster Standalone | Exact non-empty `*.svc.cluster.local` HTTP or HTTPS URL | Read the mounted token for the calling Pod's configured KSA before every request. |
| `ManagedKsa` | HTTPS Service DNS, CA path, and fixed audience-bound token path | Planned explicit profile. Read the opaque token before every request and fail before transport if a required input is missing. |

An in-cluster Standalone application uses the standard mounted token:

| Setting | Value |
|---|---|
| Audience | Kubernetes API server default |
| File | `/var/run/secrets/kubernetes.io/serviceaccount/token` |
| Rotation owner | Kubelet |

`lumen spec gen` composes the generic file-bearer provider from
`libs/openapi-codegen` with this Lumen-owned path and host rule. Each generated
TypeScript, Python, and Rust client reads the current file before it calls the
transport. The client does not interpret the token as proof of identity.
Server-side TokenReview is the canonical verifier for signature, expiry,
audience, and KSA identity.

The decision rules are:

| File state | Client behavior |
|---|---|
| Explicit Authorization header | Preserve it and do not read the token path. |
| Local, Docker, IP, or external URL | Do not read the path and send no generated Authorization header. |
| Eligible browser URL | Reject before transport. Browsers do not read the file. |
| Eligible token path is missing, unreadable, empty, or invalid | Reject the request before admission and transport. |
| Eligible token path contains valid bytes | Send them as `Authorization: Bearer <token>`. |
| Server returns `401` | Return an authentication error. Never retry as anonymous. |

The portable client treats the token as an opaque credential. The current Rust
`ProjectedTokenFile` can perform local JWT preflight, but that check never
replaces TokenReview and is not the portable generated-client contract. No
token value may enter an argument, environment variable, Fleet object, status,
Event, log, or error.

The future ManagedKsa profile keeps a different contract. It uses HTTPS,
private CA trust, audience `lumen.axiom.dev`, and the fixed path
`/var/run/secrets/lumen.axiom.dev/token`. The planned versioned
client-workload template sets
`automountServiceAccountToken: false`, mounts the audience-specific projection,
and mounts the operator-published CA ConfigMap without `subPath`. It uses an
existing ServiceAccount. It does not create a namespace, KSA, client
Deployment, or token Secret. See [client integration](client-integration.md).

OpenAPI describes the bearer header only. It does not obtain the KSA token.
`libs/openapi-codegen` owns the generic request-time provider hook. It does not
own the KSA path, audience, Fleet access policy, or Kubernetes RBAC.

## Human developer access

**Current.** `lumen connect` uses the developer's kubeconfig identity to call
TokenRequest for a named ServiceAccount. It refreshes the short-lived token and
presents it through a loopback proxy. The developer also supplies the private
serving CA when the target uses private TLS.

Who may create a token for a ServiceAccount is a Kubernetes RBAC decision. The
current `lumen k8s access render` command can render an issuer grant. Fleet does
not manage the human issuer permission in the planned model.

## Anonymous endpoints

**Current.** When auth is required, the router leaves these paths outside the
auth middleware: `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, `/docs`,
`/version`, and `/debug/cluster`.

**Planned.** Only these endpoints remain anonymous:

- `/healthz`
- `/readyz`
- `/metrics`
- `/version`

Every other runtime endpoint is protected. This includes `/debug/cluster`,
`/openapi.json`, and `/docs`.

## Migration contract

**Planned.** The 0.4.x compatibility window follows these rules:

| Input | 0.4.x migration behavior |
|---|---|
| `access` is absent | Keep the current external RBAC behavior and emit a deprecation. |
| `access.allowedServiceAccounts` is empty | Converge a valid explicit deny-all policy. |
| Current per-collection grants | Keep working and mark them deprecated. |
| `auth: disabled` in Managed | Keep working and mark it deprecated. |

Version 0.5.0 requires an explicit Managed `access` value. It removes the
per-collection grant model and rejects `auth: disabled` in Managed mode.

These migration warnings and 0.5.0 refusals are not implemented in the current
source.

## Responsibility boundaries

| Source | Responsibility |
|---|---|
| `apps/lumen` | Standalone and Managed modes, typed access CRD, whole-runtime permission meaning, anonymous route policy, Lumen audience and path, and SAR resource mapping. |
| `libs/service-auth` | Current Rust projected-token preflight, a planned portable opaque token source, TokenRequest, TokenReview, SubjectAccessReview, redaction, and fail-closed decisions. TokenReview remains the server authority. |
| `libs/openapi-codegen` | A service-neutral provider that can return dynamic request headers before every request. |
| `libs/service-k8s` | Projected-token rendering, Role and RoleBinding shapes, certificate lifecycle wiring, public CA ConfigMap ownership, prune, and status projection. |
| Kubernetes | ServiceAccount identity, short-lived token issue and rotation, RBAC policy, TokenReview, and SubjectAccessReview results. |

## References

- [Kubernetes Service Accounts](https://kubernetes.io/docs/concepts/security/service-accounts/)
- [Kubernetes projected volumes](https://kubernetes.io/docs/concepts/storage/projected-volumes/)
- [Kubernetes RBAC good practices](https://kubernetes.io/docs/concepts/security/rbac-good-practices/)
- [OpenAPI Security Scheme Object](https://spec.openapis.org/oas/latest.html#security-scheme-object)
