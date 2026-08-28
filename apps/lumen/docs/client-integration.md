# Integrate a Lumen client

## Purpose

This guide owns the boundary between Lumen, generated API clients, Kubernetes
workload manifests, and caller-owned source code. It describes current behavior
and the target best-practice client integration. Planned behavior is not an
installation step until STATUS promotes it.

Lumen minimizes application work that belongs to a search service. It cannot
take ownership of source-database credentials, source authorization, CDC
checkpoints, or final source-record hydration.

## Contract map

| Fact | Canonical source | Discovery |
|---|---|---|
| HTTP methods, paths, schemas, declared status, media types, and security | Lumen OpenAPI | `lumen spec` or [`clients/openapi.json`](../clients/openapi.json) |
| Current language generation and limits | [Generated clients](../clients/README.md) | `lumen spec gen --help` |
| Current support and material limits | [Lumen status](../STATUS.md#support-matrix) | Read the generated-client and Managed-auth rows. |
| Future auth, retry, template, and helper outcomes | [Lumen roadmap](../ROADMAP.md) | Follow the outcome linked by each unsupported STATUS row. |
| KSA identity and runtime access | [Authentication](authentication.md) | Read current and planned flows separately. |
| Source selection and hydration | [Querying](querying.md) | Use the ordered-ID flow. |

## Responsibility boundary

| Owner | Responsibility |
|---|---|
| Lumen server | Search schema, index mutation, query, filter, sort, limit, cursor, facets, request authorization, rebuild, backup, and restore. |
| Generated Lumen client | Request encoding, explicit connection profile, request-time auth provider, typed errors, stream handling, protocol fallback, safe retry, idempotency input, cursor types, and ordered-result helpers. |
| Versioned client Kustomize template | Select the app's existing KSA, project a Lumen-audience token, mount the public CA bundle, expose non-secret connection paths, and offer a narrow egress policy. |
| Caller or ingest adapter | Source writes, CDC or outbox checkpoint, source-to-index mapping, source ACL, raw vector and perceptual-hash production, bulk ID lookup, and final hydration. |
| Kubernetes and platform | KSA issue and rotation, RBAC results, client namespace, cluster networking, issuer, trust policy, and Google API identity. |

An attached KSA does not add an HTTP Authorization header. The Pod template
must project a token for Lumen's audience. The API client must read that file
and add the header.

The operator does not mutate a client Deployment. Hidden admission injection
is not part of the target. The application owns when its Pod template adopts a
new Lumen client template version.

## Connection profiles

The target generated clients use an explicit connection profile. They do not
infer the security mode from whether one conventional file happens to exist.

### Standalone

- Local development normally uses `http://127.0.0.1:7373` and reads no credential.
- In-cluster generated Lumen clients use `http://lumen.lumen.svc.cluster.local:7373` and opt into the Kubernetes default ServiceAccount token at `/var/run/secrets/kubernetes.io/serviceaccount/token`.
- The provider rereads the token once per request, only for exact non-empty `.svc.cluster.local` HTTP or HTTPS hosts.
- No private CA is required.
- A caller must opt into a different listener explicitly.

### ManagedKsa

- Base URL uses `https://<runtime>.<namespace>.svc:7373`.
- Hostname verification uses the exact Service DNS name.
- The public CA path is required.
- The projected token path is required.
- A missing, unreadable, or empty token file fails before transport.
- The client reads the current token again before every request.
- A 401 never falls back to anonymous access.

The standard target token path is:

```text
/var/run/secrets/lumen.axiom.dev/token
```

The token is opaque to the portable client contract. Kubernetes TokenReview is
the canonical verifier for signature, expiration, audience, bound identity,
and ServiceAccount shape. A local diagnostic check does not replace the server
decision.

ManagedKsa is not the Standalone in-cluster profile. Its private audience,
HTTPS, and private-CA requirements remain unchanged.

## Generated client behavior

Lumen continues to deliver generated source rather than published npm, PyPI,
or crates.io packages. A consumer can vendor the output.

The target output manifest records:

- OpenAPI SHA-256;
- generator version;
- selected language target;
- Lumen compatibility version; and
- every runtime dependency required by the emitted source.

The target generated runtime owns behavior that application code should not
reimplement:

- dedicated `QUERY` use with permanent POST fallback;
- typed Lumen API errors that retain status, headers, and unknown bodies;
- `Retry-After` handling;
- bounded exponential backoff with jitter;
- caller deadline and cancellation;
- NDJSON request and response streaming;
- one request-time dynamic header provider;
- automatic idempotency-key creation when the operation contract permits it;
- typed cursor and result contracts; and
- an ordered hydration helper.

Retry is operation-aware:

- reads and the semantic POST query fallback can retry within the caller's
  deadline;
- a write can retry only when the server's durable `Idempotency-Key` contract
  applies;
- an ambiguous write without that contract returns an error to the caller;
- 429 honors `Retry-After`;
- documented retryable 503 responses use bounded backoff; and
- a token-rotation race can retry only a safe read or an idempotent write.

These behaviors are target outcomes. Current generated clients still have
static auth inputs, incomplete error decoding, no NDJSON method, and no common
retry runtime.

## Kubernetes workload template

The planned client template is a versioned, copy-to-customize starting point.
It is separate from `k8s/overlays/template`, which is the current Standalone
runtime compatibility template.

The future client template will require an existing application workload and
KSA. It will make these changes explicit:

```yaml
spec:
  template:
    spec:
      serviceAccountName: REPLACE_ME__CLIENT_KSA
      automountServiceAccountToken: false
      volumes:
        - name: lumen-identity
          projected:
            sources:
              - serviceAccountToken:
                  audience: lumen.axiom.dev
                  expirationSeconds: 3600
                  path: token
        - name: lumen-trust
          configMap:
            name: REPLACE_ME__RUNTIME_CA_CONFIGMAP
      containers:
        - name: REPLACE_ME__CONTAINER
          env:
            - name: LUMEN_URL
              value: https://REPLACE_ME__RUNTIME.REPLACE_ME__NAMESPACE.svc:7373
            - name: LUMEN_CA_FILE
              value: /var/run/lumen/trust/ca.crt
          volumeMounts:
            - name: lumen-identity
              mountPath: /var/run/secrets/lumen.axiom.dev
              readOnly: true
            - name: lumen-trust
              mountPath: /var/run/lumen/trust
              readOnly: true
```

The actual template will also include placeholder validation and an optional
egress NetworkPolicy. It will not use `subPath` for the CA mount, because that
would prevent in-place ConfigMap updates from reaching the container.

The operator will publish one runtime-specific public CA ConfigMap into every
namespace that contains an allowed KSA. Multiple allowed KSAs in the same
namespace share that bundle. The ConfigMap contains no token, private key, or
client credential.

The template itself is not present today. Do not copy the example above as if
it were a supported artifact. Until the roadmap outcome lands, the application
platform must build and verify its own equivalent patch.

## Source integration

PostgreSQL or another source store remains authoritative. The normal request
flow is:

1. The caller writes the source record.
2. A caller-owned CDC or outbox adapter sends derived searchable values to
   Lumen.
3. Lumen performs business filter, ranking, sort, limit, cursor, and facet
   work.
4. Lumen returns ordered external IDs and search metadata.
5. The caller performs one bulk source lookup by ID list.
6. The generated helper restores Lumen's result order and joins search
   metadata to the hydrated records.

The helper never connects to PostgreSQL. It accepts the ordered hits and a
caller-supplied bulk fetch function or returned ID map. This keeps source
credentials, row-level authorization, projection, and transaction policy out
of the search client while avoiding N+1 lookups and handwritten reorder code.

Lumen does not run an embedding model. A caller or ingest pipeline may supply
a raw vector. Lumen keeps vector indexing, kNN, RRF, and filter behavior. The
same boundary applies to perceptual hashes: the caller supplies the hash, and
Lumen performs Hamming-distance search.

## Failure handling

Failures stay with the component that can decide them correctly:

| Failure | Owner and behavior |
|---|---|
| Source write, CDC checkpoint, or outbox replay | Caller keeps the source event until the Lumen write contract acknowledges it. |
| Network error on a read | Generated client applies bounded retry inside the deadline. |
| Ambiguous write | Generated client retries only with a valid idempotency key; otherwise it returns the ambiguity. |
| 429 | Generated client honors `Retry-After`. |
| Retryable routed 503 | Generated client uses bounded exponential backoff with jitter. |
| Missing Managed token or CA file | Managed connection fails before transport. It never switches to Standalone. |
| Token rejected by TokenReview | Server returns 401. The client does not downgrade to anonymous. |
| Source ID no longer exists | Caller decides whether to omit, tombstone, repair, or report the stale index entry. |
| Source authorization rejects a record | Caller omits or rejects it according to the source policy. Lumen has no source-record ACL. |

No credential value may enter an argument, environment variable, generated
manifest, status, Event, log, or error message.

## Verification

Current generated-source and snapshot gates are listed in the
[generated-client guide](../clients/README.md#verification).

Future completion evidence must prove all three generated languages without a
silent skip. It must cover:

- explicit Standalone and ManagedKsa profiles;
- token replacement between two requests;
- missing, unreadable, and empty Managed token files;
- private CA and Service DNS verification;
- no anonymous fallback after 401;
- safe read retry, `Retry-After`, bounded 503 retry, deadline, and cancellation;
- idempotent write replay and ambiguous non-idempotent write refusal;
- NDJSON streaming and typed error preservation;
- one bulk hydration call with stable result ordering;
- complete target dependency manifests; and
- redaction of token-shaped input from every captured output.

The future Kustomize template gate must run `kubectl kustomize`, reject every
remaining placeholder, and pass server-side dry-run against the supported GKE
version. It must show that the default Kubernetes API token is not mounted,
the Lumen-audience token and CA are mounted without token-valued environment
variables, and CA ConfigMap rotation reaches the file.

## Current boundaries

- TypeScript and Python retain construction-time static auth values.
- Generated TypeScript, Python, and Rust Lumen clients also opt into the
  Kubernetes default ServiceAccount token for eligible in-cluster Service DNS
  requests. ManagedKsa remains a separate private-audience profile.
- No generated client workload template or published SDK package exists.
- No generated client provides the complete safe-retry behavior above.
- No generated client exposes the NDJSON reindex stream or one typed Lumen API
  error.
- Cross-language gates can currently skip a missing toolchain.
- No generated ordered hydration helper exists.
- No versioned client workload Kustomize template exists.
- Current Fleet does not create access RBAC or publish client CA ConfigMaps.
- Generated source is not published as a language package.

## Supporting documents

- [Lumen README](../README.md)
- [Generated clients](../clients/README.md)
- [Protocol](protocol.md)
- [Indexing](indexing.md)
- [Querying](querying.md)
- [Authentication](authentication.md)
- [GKE](gke.md)
- [Deployment](deployment.md)
- [Current support](../STATUS.md)
- [Future outcomes and non-goals](../ROADMAP.md)
