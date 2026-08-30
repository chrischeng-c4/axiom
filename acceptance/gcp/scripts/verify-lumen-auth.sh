#!/usr/bin/env bash
# Two-hop KSA/RBAC authorization on GKE (#2879).
#
# #2868 deleted the old auth leg rather than repointing it: the identity it
# proved — a long-lived token Secret projected by the CSI driver — no longer
# exists in any form. This is the replacement, and it proves the shape the
# epic actually specifies.
#
#   Hop 1  a Google principal (a human account, a GSA) authenticates to
#          kube-apiserver and asks it for a TokenRequest on ONE named client
#          ServiceAccount. That is the only place a Google credential is used.
#   Hop 2  the returned short-lived, `lumen.axiom.dev`-audience KSA token is
#          the only thing sent to Lumen. Lumen validates it with TokenReview
#          and authorizes it with SubjectAccessReview.
#
# What makes the leg mean something is the second issuer. The human account on
# this cluster is cluster-admin, so every RBAC denial row would pass against it
# for the wrong reason. `LUMEN_AUTH_ISSUER_GSA` names a service account holding
# NO project IAM role, reachable by impersonation without a key file, whose
# only path to a token is the Role `lumen k8s access render` emits — so a
# denial it reports is a denial RBAC actually made.
set -euo pipefail

: "${PROJECT_ID:?PROJECT_ID is required}"
: "${RUN_ID:?RUN_ID is required}"
: "${EVIDENCE_DIR:?EVIDENCE_DIR is required}"
: "${LUMEN_CLI:?LUMEN_CLI is required}"
: "${LUMEN_AUTH_ISSUER_GSA:?LUMEN_AUTH_ISSUER_GSA is required; use a pre-provisioned least-privilege issuer GSA}"

ISSUER_GSA="$LUMEN_AUTH_ISSUER_GSA"

AUTH_NS=lumen
CLIENT_NS=lumen-auth-client
INSTANCE=lumen-auth
PORT=17375
GRANTED=authz
UNGRANTED=authz-other
AUDIENCE=lumen.axiom.dev
# `service_auth::k8s::CachePolicy::default()` — allow_ttl 300s + stale_window
# 60s. `apps/lumen/src/auth.rs` takes the default, so this is the documented
# bound a revoked SubjectAccessReview must become effective within (R7).
REVOCATION_BOUND_SECONDS=360
identity_observations=0
non_ksa_rejections=0
authorization=0
sibling_refusals=0
revocations=0
redaction=0
teardown=0
probe_category=authorization

AUTH_EVIDENCE="$EVIDENCE_DIR/kubernetes/auth"
mkdir -p "$AUTH_EVIDENCE"

# Credentials live here and never in $EVIDENCE_DIR (R8). One directory, one
# trap, 0700 — so there is a single place to be sure about.
SECRET_DIR="$(mktemp -d "${TMPDIR:-/tmp}/lumen-auth-${RUN_ID}.XXXXXX")"
chmod 700 "$SECRET_DIR"

forward_pid=""

stop_forward() {
  if [[ -n "$forward_pid" ]]; then
    kill "$forward_pid" >/dev/null 2>&1 || true
    wait "$forward_pid" >/dev/null 2>&1 || true
    forward_pid=""
  fi
}

cleanup_secrets() {
  stop_forward
  rm -rf "$SECRET_DIR"
}
trap cleanup_secrets EXIT INT TERM

start_forward() {
  stop_forward
  local deadline=$((SECONDS + 120))
  while (( SECONDS < deadline )); do
    if [[ -z "$forward_pid" ]] || ! kill -0 "$forward_pid" >/dev/null 2>&1; then
      stop_forward
      kubectl -n "$AUTH_NS" port-forward "service/$INSTANCE" "$PORT:7373" \
        >>"$AUTH_EVIDENCE/port-forward.log" 2>&1 &
      forward_pid="$!"
      sleep 1
    fi
    # /readyz is on the probe router, outside the auth middleware — it answers
    # without a credential by design, which is exactly what makes it usable as
    # a readiness signal for an instance that requires one.
    if curl --max-time 5 --silent --show-error --fail \
      "http://127.0.0.1:$PORT/readyz" >/dev/null 2>&1; then
      return 0
    fi
    sleep 2
  done
  echo "timed out waiting for $INSTANCE readiness through port-forward" >&2
  return 1
}

fail() {
  echo "$*" >&2
  kubectl -n "$AUTH_NS" get lumen/"$INSTANCE" -o yaml >&2 || true
  kubectl -n "$AUTH_NS" logs "statefulset/$INSTANCE" --tail=200 >&2 || true
  exit 1
}

# ---------------------------------------------------------------------------
# Phase 0 — observe both issuer principals
# ---------------------------------------------------------------------------
# R2 is explicit that the Kubernetes usernames must be OBSERVED, not derived.
# A Google address reaches the authorizer as whatever the cluster's
# authenticator produced; guessing at normalization would bind a RoleBinding to
# a user nobody is, and every denial row would then pass for the wrong reason.
echo ">> observing issuer principals"

kubectl auth whoami -o json > "$AUTH_EVIDENCE/issuer-human-whoami.json"
HUMAN_USER="$(jq -r '.status.userInfo.username' "$AUTH_EVIDENCE/issuer-human-whoami.json")"
[[ -n "$HUMAN_USER" && "$HUMAN_USER" != "null" ]] || fail "could not observe the ambient kubeconfig's Kubernetes username"
identity_observations=$((identity_observations + 1))

resolve_gsa_issuer() {
  if ! gcloud auth print-access-token \
    --impersonate-service-account="$ISSUER_GSA" > "$SECRET_DIR/gsa-access-token" 2>"$AUTH_EVIDENCE/issuer-gsa-impersonation-error.txt"; then
    cat >&2 <<EOF
cannot impersonate the least-privilege issuer $ISSUER_GSA.

This leg needs a pre-provisioned second Kubernetes principal that is NOT
cluster-admin and is impersonable by the runner. The harness never provisions
GCP identities or IAM bindings.
EOF
    exit 1
  fi
  chmod 600 "$SECRET_DIR/gsa-access-token"
}
resolve_gsa_issuer

# A kubeconfig whose only credential is that access token: same cluster, same
# CA, a different user. Built by rewriting the minified ambient config rather
# than by `set-credentials`, because kubectl refuses a token beside the exec
# block GKE's context already carries.
kubectl config view --raw --minify -o json > "$SECRET_DIR/base-kubeconfig.json"
jq --rawfile token "$SECRET_DIR/gsa-access-token" '
  .users = [{name: "gsa-issuer", user: {token: ($token | sub("\\s+$"; ""))}}]
  | .contexts = [(.contexts[0] | .name = "gsa-issuer" | .context.user = "gsa-issuer")]
  | ."current-context" = "gsa-issuer"
' "$SECRET_DIR/base-kubeconfig.json" > "$SECRET_DIR/gsa.kubeconfig"
chmod 600 "$SECRET_DIR/gsa.kubeconfig"

KUBECONFIG="$SECRET_DIR/gsa.kubeconfig" kubectl auth whoami -o json \
  > "$AUTH_EVIDENCE/issuer-gsa-whoami.json" \
  || fail "$ISSUER_GSA authenticated to GCP but not to kube-apiserver"
GSA_USER="$(jq -r '.status.userInfo.username' "$AUTH_EVIDENCE/issuer-gsa-whoami.json")"
[[ -n "$GSA_USER" && "$GSA_USER" != "null" ]] || fail "could not observe $ISSUER_GSA's Kubernetes username"
identity_observations=$((identity_observations + 1))

# Which issuers are meaningful subjects for a denial row. `can-i '*' '*'` is
# the whole question: a cluster-admin is denied nothing, so a denial it reports
# would be evidence of nothing.
HUMAN_ADMIN="$(kubectl auth can-i '*' '*' --all-namespaces 2>/dev/null || true)"
GSA_ADMIN="$(KUBECONFIG="$SECRET_DIR/gsa.kubeconfig" kubectl auth can-i '*' '*' --all-namespaces 2>/dev/null || true)"
[[ "$GSA_ADMIN" == "yes" ]] && fail "$GSA_USER is cluster-admin; it cannot serve as the least-privilege issuer"

printf 'human=%s cluster_admin=%s\ngsa=%s cluster_admin=%s\n' \
  "$HUMAN_USER" "$HUMAN_ADMIN" "$GSA_USER" "$GSA_ADMIN" \
  > "$AUTH_EVIDENCE/issuers.txt"
echo ">> issuers: $HUMAN_USER (cluster-admin=$HUMAN_ADMIN), $GSA_USER (cluster-admin=$GSA_ADMIN)"

# ---------------------------------------------------------------------------
# Phase 1 — an instance that requires an identity
# ---------------------------------------------------------------------------
main_cr_image="$(kubectl -n "$AUTH_NS" get lumen/lumen -o jsonpath='{.spec.image}')"
[[ -n "$main_cr_image" ]] || fail "could not read the main lumen CR's image for the auth instance"
# Borrow the main CR's raftStorage rather than naming a size here: the volume
# size that provisions on this cluster is a property of its StorageClass, not
# of this leg, and a hardcoded one would make the auth proof fail on a disk
# request the persistence leg already knows how to satisfy.
main_cr_raft_storage="$(kubectl -n "$AUTH_NS" get lumen/lumen -o jsonpath='{.spec.serving.raftStorage}')"
[[ -n "$main_cr_raft_storage" ]] || fail "could not read the main lumen CR's serving.raftStorage"

# Inherit `spec.placement` from the main CR verbatim, as JSON -- YAML is a
# superset of JSON, so the block below embeds without enumerating keys and
# cannot drift from what the placement leg already proved.
#
# This is load-bearing, not cosmetic. Since 0.4.26 the operator resolves a
# machine profile out of the `lumen-system/lumen-capacity-catalog` ConfigMap
# that `apps/lumen/terraform/modules/lumen-capacity/catalog.tf` publishes, and
# nothing in this harness applies that module -- the ConfigMap has never
# existed in the acceptance cluster. 0.4.28 added the escape hatch that makes
# every other leg work anyway (apps/lumen/src/operator/reconcile.rs:815): a
# NON-EMPTY `placement.nodeSelector` with the default `initialMachineType`
# takes the self-contained Kubernetes-native path and never reads the catalog.
# `render-manifests.sh:55` renders the main CR with `--profile dev`, and the
# dev renderer supplies `kubernetes.io/os: linux` (apps/lumen/src/bin/lumen.rs:2723),
# so the main instance qualifies. A CR authored here without a placement block
# does not: it falls to the legacy catalog path, every reconcile fails with
# `configmaps "lumen-capacity-catalog" not found`, no StatefulSet is ever
# created, and the leg dies as `never reached Ready` with the actual reason
# only in the operator log.
main_cr_placement="$(kubectl -n "$AUTH_NS" get lumen/lumen -o jsonpath='{.spec.placement}')"
[[ -n "$main_cr_placement" ]] || fail "could not read the main lumen CR's spec.placement"
# An empty selector is the exact condition that drops this CR onto the catalog
# path, so refuse it here by name rather than 10 minutes later as a timeout.
echo "$main_cr_placement" | jq -e '.nodeSelector | length > 0' > /dev/null \
  || fail "the main lumen CR's spec.placement.nodeSelector is empty; $INSTANCE would fall to the legacy capacity-catalog path this harness never provisions"

echo ">> applying $INSTANCE with auth: required"
cat <<EOF | kubectl apply -f - > "$AUTH_EVIDENCE/cr-apply.txt"
apiVersion: lumen.dev/v1alpha1
kind: Lumen
metadata:
  name: ${INSTANCE}
  namespace: ${AUTH_NS}
spec:
  image: ${main_cr_image}
  imagePullPolicy: IfNotPresent
  shardCount: 1
  replicasPerShard: 1
  voterCount: 1
  logFormat: json
  # The one CR in this harness that does NOT opt out. AuthMode::Off serializes
  # as \`disabled\`, never \`off\` — YAML 1.1 would read the latter as boolean
  # false — and \`required\` is the default the other legs override.
  auth: required
  placement: ${main_cr_placement}
  serving:
    cpu: 500m
    memory: 1Gi
    raftStorage: ${main_cr_raft_storage}
EOF

# A serving process under `auth: required` refuses to start when the delegated
# review probe fails, so "never reached Ready" is the expected shape of an auth
# wiring defect -- and the pod that holds the reason dies with the namespace the
# cleanup trap tears down seconds later. Capture the diagnosis before failing,
# or the only artifact of the failure is the sentence "never reached Ready".
capture_unready_evidence() {
  kubectl -n "$AUTH_NS" get lumen/"$INSTANCE" -o json \
    > "$AUTH_EVIDENCE/cr-unready.json" 2>&1 || true
  kubectl -n "$AUTH_NS" describe pod "${INSTANCE}-0" \
    > "$AUTH_EVIDENCE/pod-unready-describe.txt" 2>&1 || true
  kubectl -n "$AUTH_NS" logs "${INSTANCE}-0" --tail=200 \
    > "$AUTH_EVIDENCE/pod-unready.log" 2>&1 || true
  kubectl -n "$AUTH_NS" logs "${INSTANCE}-0" --previous --tail=200 \
    > "$AUTH_EVIDENCE/pod-unready-previous.log" 2>&1 || true
  kubectl get clusterrolebinding "lumen.${AUTH_NS}.${INSTANCE}.auth-delegator" -o json \
    > "$AUTH_EVIDENCE/auth-delegator-binding-unready.json" 2>&1 || true
}

auth_ready_deadline=$((SECONDS + 600))
until [[ "$(kubectl -n "$AUTH_NS" get lumen/"$INSTANCE" -o jsonpath='{.status.phase}' 2>/dev/null || true)" == "Ready" ]]; do
  # A container that has already died twice is not slow, it is broken, and the
  # backoff only grows from here. Waiting out the full deadline would add ten
  # minutes to a verdict the second restart already settled.
  restarts="$(kubectl -n "$AUTH_NS" get pod "${INSTANCE}-0" \
    -o jsonpath='{.status.containerStatuses[?(@.name=="server")].restartCount}' 2>/dev/null || true)"
  if [[ "${restarts:-0}" =~ ^[0-9]+$ ]] && (( restarts >= 2 )); then
    capture_unready_evidence
    fail "${INSTANCE}-0 restarted ${restarts} times without serving \
(see $AUTH_EVIDENCE/pod-unready-previous.log)"
  fi
  if (( SECONDS >= auth_ready_deadline )); then
    capture_unready_evidence
    fail "$INSTANCE never reached Ready (see $AUTH_EVIDENCE/pod-unready*.log)"
  fi
  sleep 5
done
kubectl -n "$AUTH_NS" get lumen/"$INSTANCE" -o json > "$AUTH_EVIDENCE/cr-after-apply.json"

# Readiness alone would not prove delegation is wired: `LumenVerifier::connect`
# probes both grants at startup and refuses to serve without
# `system:auth-delegator`, so a Ready pod already implies the binding — but the
# object is cluster-scoped and swept on a separate path, so assert it directly.
delegator="lumen.${AUTH_NS}.${INSTANCE}.auth-delegator"
kubectl get clusterrolebinding "$delegator" -o json > "$AUTH_EVIDENCE/auth-delegator-binding.json" \
  || fail "the operator did not render the delegated-review ClusterRoleBinding $delegator"
jq -e --arg ns "$AUTH_NS" --arg sa "$INSTANCE" '
  .roleRef.name == "system:auth-delegator"
  and (.subjects | length) == 1
  and .subjects[0].kind == "ServiceAccount"
  and .subjects[0].namespace == $ns
  and .subjects[0].name == $sa
' "$AUTH_EVIDENCE/auth-delegator-binding.json" >/dev/null \
  || fail "$delegator does not bind exactly the serving ServiceAccount to system:auth-delegator"

start_forward

# ---------------------------------------------------------------------------
# Phase 2 — the client ServiceAccounts and their grants
# ---------------------------------------------------------------------------
# Every grant below is rendered by `lumen k8s access render` (#2889), not
# hand-written: the resource names and verbs the SubjectAccessReview asks about
# come from `lumen::auth`, so a hand-written Role could describe a check Lumen
# never makes and the matrix would prove nothing about the product.
render_access() {
  local namespace="$1" client="$2"
  shift 2
  "$LUMEN_CLI" k8s access render \
    --namespace "$namespace" --client-sa "$client" \
    --issuer "$HUMAN_USER" --issuer "$GSA_USER" \
    "$@" > "$AUTH_EVIDENCE/access-${namespace}-${client}.yaml"
  kubectl apply -f "$AUTH_EVIDENCE/access-${namespace}-${client}.yaml" \
    > "$AUTH_EVIDENCE/access-${namespace}-${client}-apply.txt"
}

kubectl create namespace "$CLIENT_NS" > "$AUTH_EVIDENCE/client-namespace.txt"

# `--grant` is cumulative (`granted_verbs`): write covers get+update, admin
# covers get+update+delete. So the interesting AC4 rows are the ones a level
# does NOT reach — and `lumenadmin` is reached by no collection grant at all.
render_access "$AUTH_NS" auth-reader   --grant "$GRANTED=read"
render_access "$AUTH_NS" auth-writer   --grant "$GRANTED=write"
render_access "$AUTH_NS" auth-admin    --grant "$GRANTED=admin" --grant "$UNGRANTED=admin"
render_access "$AUTH_NS" auth-operator --instance-admin
render_access "$AUTH_NS" auth-unbound  --grant "$GRANTED=read"
render_access "$CLIENT_NS" auth-foreign --grant "$GRANTED=read"

# AC5 wants an account that authenticates and is authorized for nothing. Drop
# only the Lumen half of its bundle: the issuer Role stays, so the token is
# still mintable and the 403 is about authorization rather than about minting.
kubectl -n "$AUTH_NS" delete rolebinding auth-unbound-lumen-access \
  > "$AUTH_EVIDENCE/unbound-rolebinding-deleted.txt"

# AC6's target: a ServiceAccount that exists and that no issuer Role names.
kubectl -n "$AUTH_NS" create serviceaccount auth-sibling \
  > "$AUTH_EVIDENCE/sibling-serviceaccount.txt"

# ---------------------------------------------------------------------------
# Phase 3 — hop 1: TokenRequest, once per issuer
# ---------------------------------------------------------------------------
# Long enough to outlive the revocation poll in phase 8, so a 401 there would
# mean expiry and a 403 would mean revocation — two different findings that a
# 10-minute token would blur into one.
TOKEN_DURATION=45m

mint() {
  local issuer="$1" sa="$2" namespace="${3:-$AUTH_NS}"
  # Spelled out per branch rather than through an array of env assignments:
  # bash 3.2 expands an empty array under `set -u` as an unbound variable, and
  # this ships on the macOS the acceptance run is driven from.
  if [[ "$issuer" == "gsa" ]]; then
    KUBECONFIG="$SECRET_DIR/gsa.kubeconfig" kubectl create token "$sa" -n "$namespace" \
      --audience "$AUDIENCE" --duration "$TOKEN_DURATION"
  else
    kubectl create token "$sa" -n "$namespace" \
      --audience "$AUDIENCE" --duration "$TOKEN_DURATION"
  fi
}

echo ">> minting client tokens through both issuers"
for issuer in human gsa; do
  for sa in auth-reader auth-writer auth-admin auth-operator auth-unbound; do
    mint "$issuer" "$sa" > "$SECRET_DIR/${issuer}-${sa}.token" \
      || fail "$issuer could not mint a token for $sa, which its rendered grant permits"
    chmod 600 "$SECRET_DIR/${issuer}-${sa}.token"
  done
  mint "$issuer" auth-foreign "$CLIENT_NS" > "$SECRET_DIR/${issuer}-auth-foreign.token" \
    || fail "$issuer could not mint a token for auth-foreign in $CLIENT_NS"
  chmod 600 "$SECRET_DIR/${issuer}-auth-foreign.token"
done

# AC6 — the issuer's Role names one ServiceAccount, so a sibling is refused.
# Asserted for every issuer that is not cluster-admin; a cluster-admin is
# denied nothing, so asserting it there would be asserting nothing. The leg
# already refused to start without at least one least-privilege issuer.
sibling_refusals=0
for issuer in human gsa; do
  admin_verdict="$HUMAN_ADMIN"
  [[ "$issuer" == "gsa" ]] && admin_verdict="$GSA_ADMIN"
  if [[ "$admin_verdict" == "yes" ]]; then
    echo "note: $issuer is cluster-admin; sibling-mint refusal is not assertable against it" \
      >> "$AUTH_EVIDENCE/sibling-mint.txt"
    continue
  fi
  if mint "$issuer" auth-sibling > "$SECRET_DIR/sibling-attempt.token" 2>>"$AUTH_EVIDENCE/sibling-mint.txt"; then
    rm -f "$SECRET_DIR/sibling-attempt.token"
    fail "$issuer minted a token for auth-sibling; the issuer Role's resourceNames did not bind"
  fi
  echo "$issuer refused a TokenRequest for auth-sibling" >> "$AUTH_EVIDENCE/sibling-mint.txt"
  sibling_refusals=$((sibling_refusals + 1))
done
rm -f "$SECRET_DIR/sibling-attempt.token"
(( sibling_refusals >= 1 )) || fail "no least-privilege issuer proved the sibling-mint refusal (AC6)"

# ---------------------------------------------------------------------------
# The probe. One function, so every row is the same request shaped by
# arguments — a row that passes for a different reason than the row above it
# is the failure mode this leg exists to rule out.
# ---------------------------------------------------------------------------
probe() {
  local label="$1" expected="$2" token_file="$3" method="$4" path="$5" data="${6:-}"
  local args=(--silent --show-error --max-time 20
    -o "$AUTH_EVIDENCE/probe-${label}.body.json"
    -w '%{http_code}'
    -X "$method" "http://127.0.0.1:$PORT$path")
  if [[ "$token_file" != "-" ]]; then
    args+=(-H "@$token_file")
  fi
  if [[ -n "$data" ]]; then
    args+=(-H 'content-type: application/json' --data "$data")
  fi
  local status
  status="$(curl "${args[@]}")"
  printf '%s %s -> %s (expected %s)\n' "$method" "$path" "$status" "$expected" \
    > "$AUTH_EVIDENCE/probe-${label}.status.txt"
  if [[ "$status" != "$expected" ]]; then
    echo "auth row '$label' returned $status, expected $expected" >&2
    echo "$method $path" >&2
    cat "$AUTH_EVIDENCE/probe-${label}.body.json" >&2 || true
    exit 1
  fi
  case "$probe_category" in
    authorization) authorization=$((authorization + 1)) ;;
    non_ksa_rejections) non_ksa_rejections=$((non_ksa_rejections + 1)) ;;
    revocations) revocations=$((revocations + 1)) ;;
  esac
  echo "   $label: $status"
}

# curl reads a header from a file with `-H @file`; that keeps the token off
# every argument list and out of `ps`.
header_file() {
  local token_file="$1" out="$2"
  printf 'Authorization: Bearer %s' "$(cat "$token_file")" > "$out"
  chmod 600 "$out"
}

for f in "$SECRET_DIR"/*.token; do
  header_file "$f" "${f%.token}.header"
done

hdr() { printf '%s/%s.header' "$SECRET_DIR" "$1"; }

# ---------------------------------------------------------------------------
# Phase 4 — the admin identity creates both collections
# ---------------------------------------------------------------------------
# `PUT /collections/{id}` is `Role::Admin` on that named collection, so this is
# simultaneously the setup and AC4's positive admin row.
echo ">> collection setup through the admin client ServiceAccount"
probe admin-create-granted 200 "$(hdr gsa-auth-admin)" PUT "/collections/$GRANTED" \
  '{"fields":{"message":{"type":"keyword"}}}'
probe admin-create-ungranted 200 "$(hdr gsa-auth-admin)" PUT "/collections/$UNGRANTED" \
  '{"fields":{"message":{"type":"keyword"}}}'
probe admin-index-granted 200 "$(hdr gsa-auth-admin)" POST "/collections/$GRANTED/index" \
  "{\"items\":[{\"external_id\":\"$RUN_ID\",\"field\":\"message\",\"value\":\"authz-$RUN_ID\"}]}"
probe admin-index-ungranted 200 "$(hdr gsa-auth-admin)" POST "/collections/$UNGRANTED/index" \
  "{\"items\":[{\"external_id\":\"$RUN_ID\",\"field\":\"message\",\"value\":\"authz-other-$RUN_ID\"}]}"

SEARCH_BODY="{\"query\":{\"term\":{\"field\":\"message\",\"value\":\"authz-$RUN_ID\"}},\"limit\":10}"
INDEX_BODY="{\"items\":[{\"external_id\":\"probe-$RUN_ID\",\"field\":\"message\",\"value\":\"authz-$RUN_ID\"}]}"

# ---------------------------------------------------------------------------
# Phase 5 — the grant matrix (AC3, AC4, AC5)
# ---------------------------------------------------------------------------
echo ">> grant matrix, minted through $GSA_USER"

# read: get on one named collection, and nothing else anywhere.
probe reader-search-granted    200 "$(hdr gsa-auth-reader)" POST "/collections/$GRANTED/search"   "$SEARCH_BODY"
probe reader-stats-granted     200 "$(hdr gsa-auth-reader)" GET  "/collections/$GRANTED/stats"
probe reader-index-denied      403 "$(hdr gsa-auth-reader)" POST "/collections/$GRANTED/index"    "$INDEX_BODY"
probe reader-create-denied     403 "$(hdr gsa-auth-reader)" PUT  "/collections/$GRANTED"          '{"fields":{}}'
probe reader-admin-denied      403 "$(hdr gsa-auth-reader)" GET  "/admin/backup"
probe reader-other-collection  403 "$(hdr gsa-auth-reader)" POST "/collections/$UNGRANTED/search" "$SEARCH_BODY"

# write: get+update on the same one. Not delete, and not the admin surface.
probe writer-index-granted     200 "$(hdr gsa-auth-writer)" POST "/collections/$GRANTED/index"    "$INDEX_BODY"
probe writer-search-granted    200 "$(hdr gsa-auth-writer)" POST "/collections/$GRANTED/search"   "$SEARCH_BODY"
probe writer-create-denied     403 "$(hdr gsa-auth-writer)" PUT  "/collections/$GRANTED"          '{"fields":{}}'
probe writer-admin-denied      403 "$(hdr gsa-auth-writer)" GET  "/admin/backup"
probe writer-other-collection  403 "$(hdr gsa-auth-writer)" POST "/collections/$UNGRANTED/index"  "$INDEX_BODY"

# admin on a collection is still not admin on the instance: `lumenadmin` is a
# separate resource that no `--grant` reaches.
probe collection-admin-instance-denied 403 "$(hdr gsa-auth-admin)" GET "/admin/backup"

# --instance-admin is the other direction: the instance surface, no collection.
probe operator-admin-granted   200 "$(hdr gsa-auth-operator)" GET  "/admin/backup"
probe operator-search-denied   403 "$(hdr gsa-auth-operator)" POST "/collections/$GRANTED/search" "$SEARCH_BODY"

# AC5 — authenticates, authorized for nothing.
probe unbound-search-denied    403 "$(hdr gsa-auth-unbound)" POST "/collections/$GRANTED/search"  "$SEARCH_BODY"

# The SubjectAccessReview is scoped to the serving namespace, so a grant that
# is identical except for living in another namespace authorizes nothing here.
probe foreign-namespace-denied 403 "$(hdr gsa-auth-foreign)" POST "/collections/$GRANTED/search"  "$SEARCH_BODY"

# AC1 — the same rows through the human issuer's TokenRequest. Both principals
# reach Lumen only as the client ServiceAccount they minted.
echo ">> the same identities, minted through $HUMAN_USER"
probe human-reader-search-granted 200 "$(hdr human-auth-reader)" POST "/collections/$GRANTED/search" "$SEARCH_BODY"
probe human-reader-admin-denied   403 "$(hdr human-auth-reader)" GET  "/admin/backup"
probe human-unbound-denied        403 "$(hdr human-auth-unbound)" POST "/collections/$GRANTED/search" "$SEARCH_BODY"

# ---------------------------------------------------------------------------
# Phase 6 — everything that is not a Lumen-audience KSA token (AC2)
# ---------------------------------------------------------------------------
echo ">> credentials Lumen must refuse"
probe_category=non_ksa_rejections

# A real Google access token for the GSA. kube-apiserver authenticates it —
# that is how this script reached the cluster as $GSA_USER — and Lumen still
# refuses it, because `system:serviceaccount:<ns>:<name>` is the only principal
# shape it admits.
header_file "$SECRET_DIR/gsa-access-token" "$SECRET_DIR/google-access.header"
probe google-access-token-refused 401 "$SECRET_DIR/google-access.header" \
  POST "/collections/$GRANTED/search" "$SEARCH_BODY"

# A real Google ID token whose `aud` claim IS lumen.axiom.dev. The audience
# matches by construction, so this row can only pass because TokenReview
# refuses to validate it — not because an audience string differed.
gcloud auth print-identity-token --impersonate-service-account="$ISSUER_GSA" \
  --audiences="$AUDIENCE" > "$SECRET_DIR/google-id-token" 2>/dev/null \
  || fail "could not mint a Google ID token for $ISSUER_GSA (needed for the AC2 refusal row)"
chmod 600 "$SECRET_DIR/google-id-token"
header_file "$SECRET_DIR/google-id-token" "$SECRET_DIR/google-id.header"
probe google-id-token-refused 401 "$SECRET_DIR/google-id.header" \
  POST "/collections/$GRANTED/search" "$SEARCH_BODY"

# The human account's own Google access token — a cluster-admin at
# kube-apiserver, nobody at Lumen.
gcloud auth print-access-token > "$SECRET_DIR/human-access-token" 2>/dev/null \
  || fail "could not read the ambient Google access token (needed for the AC2 refusal row)"
chmod 600 "$SECRET_DIR/human-access-token"
header_file "$SECRET_DIR/human-access-token" "$SECRET_DIR/human-access.header"
probe human-google-token-refused 401 "$SECRET_DIR/human-access.header" \
  POST "/collections/$GRANTED/search" "$SEARCH_BODY"

# A KSA token that is authentic and bound to the wrong audience. Without the
# audience check this would authenticate as auth-admin.
kubectl create token auth-admin -n "$AUTH_NS" \
  --audience "https://kubernetes.default.svc" --duration "$TOKEN_DURATION" \
  > "$SECRET_DIR/wrong-audience.token"
chmod 600 "$SECRET_DIR/wrong-audience.token"
header_file "$SECRET_DIR/wrong-audience.token" "$SECRET_DIR/wrong-audience.header"
probe wrong-audience-refused 401 "$SECRET_DIR/wrong-audience.header" \
  POST "/collections/$GRANTED/search" "$SEARCH_BODY"

probe anonymous-refused 401 - POST "/collections/$GRANTED/search" "$SEARCH_BODY"

# ---------------------------------------------------------------------------
# Phase 7 — the product's own two-hop path (R3 through the CLI)
# ---------------------------------------------------------------------------
# The matrix above mints with kubectl so each row can name one endpoint. This
# row is the shipped surface end to end: `lumen query --client-sa` performs the
# TokenRequest itself, through the caller's kubeconfig, and sends only what it
# minted.
echo ">> lumen query --client-sa, once per issuer"
probe_category=authorization
"$LUMEN_CLI" query search --url "http://127.0.0.1:$PORT" \
  --namespace "$AUTH_NS" --client-sa auth-reader \
  --collection "$GRANTED" --term "message=authz-$RUN_ID" \
  > "$AUTH_EVIDENCE/cli-query-human.json" \
  || fail "lumen query --client-sa auth-reader failed through $HUMAN_USER"
jq -e '.total >= 1' "$AUTH_EVIDENCE/cli-query-human.json" >/dev/null \
  || fail "lumen query returned no hit through $HUMAN_USER"

KUBECONFIG="$SECRET_DIR/gsa.kubeconfig" "$LUMEN_CLI" query search \
  --url "http://127.0.0.1:$PORT" \
  --namespace "$AUTH_NS" --client-sa auth-reader \
  --collection "$GRANTED" --term "message=authz-$RUN_ID" \
  > "$AUTH_EVIDENCE/cli-query-gsa.json" \
  || fail "lumen query --client-sa auth-reader failed through $GSA_USER"
jq -e '.total >= 1' "$AUTH_EVIDENCE/cli-query-gsa.json" >/dev/null \
  || fail "lumen query returned no hit through $GSA_USER"

# The CLI refuses an account the caller may not mint, before any request goes
# out. Same refusal the local test covers, now against a real API server.
if KUBECONFIG="$SECRET_DIR/gsa.kubeconfig" "$LUMEN_CLI" query search \
  --url "http://127.0.0.1:$PORT" \
  --namespace "$AUTH_NS" --client-sa auth-sibling \
  --collection "$GRANTED" --term "message=authz-$RUN_ID" \
  > "$AUTH_EVIDENCE/cli-query-sibling.txt" 2>&1; then
  fail "lumen query minted a token for auth-sibling through $GSA_USER"
fi
grep -q -- "--subresource=token" "$AUTH_EVIDENCE/cli-query-sibling.txt" \
  || fail "the CLI's mint refusal did not name the check to run"

# ---------------------------------------------------------------------------
# Phase 8 — revoking hop 1 (R6/AC7)
# ---------------------------------------------------------------------------
# Removing the issuer RoleBinding stops NEW TokenRequests. An already-issued
# token stays valid until it expires — that is what "short-lived" buys, and
# claiming otherwise would be claiming a revocation Kubernetes does not do.
echo ">> revoking the issuer binding for auth-reader"
probe_category=revocations
kubectl -n "$AUTH_NS" delete rolebinding auth-reader-token-issuer \
  > "$AUTH_EVIDENCE/issuer-revoke.txt"

issuer_revoke_started="$SECONDS"
issuer_revoke_deadline=$((SECONDS + 120))
until ! mint gsa auth-reader > /dev/null 2>&1; do
  (( SECONDS < issuer_revoke_deadline )) \
    || fail "$GSA_USER could still mint an auth-reader token 120s after its issuer binding was removed"
  sleep 2
done
issuer_revoke_seconds=$((SECONDS - issuer_revoke_started))
revocations=$((revocations + 1))
echo "TokenRequest refused ${issuer_revoke_seconds}s after the issuer RoleBinding was deleted" \
  > "$AUTH_EVIDENCE/issuer-revocation-interval.txt"
echo "   issuer revocation effective after ${issuer_revoke_seconds}s"

# ---------------------------------------------------------------------------
# Phase 9 — revoking hop 2 (R7/AC7)
# ---------------------------------------------------------------------------
# The measured quantity is the allow cache. Make an authorized read first so a
# positive decision is definitely cached, then remove the Lumen RoleBinding and
# poll until it is not. Insisting on 403 rather than "not 200" is what keeps a
# token expiry from being read as a revocation.
echo ">> revoking the Lumen binding for auth-writer (bound: ${REVOCATION_BOUND_SECONDS}s)"
probe writer-search-before-revocation 200 "$(hdr gsa-auth-writer)" \
  POST "/collections/$GRANTED/search" "$SEARCH_BODY"

kubectl -n "$AUTH_NS" delete rolebinding auth-writer-lumen-access \
  > "$AUTH_EVIDENCE/lumen-revoke.txt"
lumen_revoke_started="$SECONDS"
lumen_revoke_deadline=$((SECONDS + REVOCATION_BOUND_SECONDS + 60))
while true; do
  status="$(curl --silent --show-error --max-time 20 \
    -o "$AUTH_EVIDENCE/probe-writer-revocation-poll.body.json" -w '%{http_code}' \
    -X POST "http://127.0.0.1:$PORT/collections/$GRANTED/search" \
    -H "@$(hdr gsa-auth-writer)" -H 'content-type: application/json' \
    --data "$SEARCH_BODY")"
  [[ "$status" == "403" ]] && break
  if [[ "$status" == "401" ]]; then
    fail "the auth-writer token stopped authenticating during the revocation poll; \
this measures token expiry, not SubjectAccessReview revocation"
  fi
  if (( SECONDS >= lumen_revoke_deadline )); then
    fail "auth-writer still authorized $((SECONDS - lumen_revoke_started))s after its Lumen RoleBinding was deleted (bound ${REVOCATION_BOUND_SECONDS}s)"
  fi
  sleep 5
done
lumen_revoke_seconds=$((SECONDS - lumen_revoke_started))
revocations=$((revocations + 1))
(( lumen_revoke_seconds <= REVOCATION_BOUND_SECONDS )) \
  || fail "revocation took ${lumen_revoke_seconds}s, past the documented ${REVOCATION_BOUND_SECONDS}s allow-cache bound"
echo "SubjectAccessReview denial effective ${lumen_revoke_seconds}s after the RoleBinding was deleted (bound ${REVOCATION_BOUND_SECONDS}s)" \
  > "$AUTH_EVIDENCE/lumen-revocation-interval.txt"
echo "   Lumen revocation effective after ${lumen_revoke_seconds}s"

# ---------------------------------------------------------------------------
# Phase 10 — who Lumen thinks was calling (AC1)
# ---------------------------------------------------------------------------
# The access log records the authenticated subject per request. If a Google
# address ever reached Lumen as an identity it would be here, so its absence is
# the assertion — and the client ServiceAccount's presence is the other half.
kubectl -n "$AUTH_NS" logs "statefulset/$INSTANCE" --tail=-1 \
  > "$AUTH_EVIDENCE/serving.log" 2>&1 || true
grep -q "system:serviceaccount:${AUTH_NS}:auth-reader" "$AUTH_EVIDENCE/serving.log" \
  || fail "the serving access log never named auth-reader as a request subject"
grep -q "system:serviceaccount:${AUTH_NS}:auth-operator" "$AUTH_EVIDENCE/serving.log" \
  || fail "the serving access log never named auth-operator as a request subject"
for principal in "$HUMAN_USER" "$GSA_USER"; do
  if grep -qF "$principal" "$AUTH_EVIDENCE/serving.log"; then
    fail "the serving access log names the Google principal $principal; Lumen saw a Google identity"
  fi
done

# ---------------------------------------------------------------------------
# Phase 11 — teardown (AC8)
# ---------------------------------------------------------------------------
# CR first, then the StatefulSet, then the PVCs: drift repair recreates a
# StatefulSet deleted while its owner still exists, and a PVC outliving both
# keeps billing as a Persistent Disk.
echo ">> tearing down the auth leg"
kubectl -n "$AUTH_NS" delete lumen/"$INSTANCE" --ignore-not-found --wait=true \
  > "$AUTH_EVIDENCE/cr-delete.txt"
kubectl -n "$AUTH_NS" delete statefulset/"$INSTANCE" --ignore-not-found \
  --cascade=foreground --wait=true >> "$AUTH_EVIDENCE/cr-delete.txt"
kubectl -n "$AUTH_NS" delete pvc -l "app.kubernetes.io/instance=$INSTANCE" \
  --ignore-not-found --wait=true >> "$AUTH_EVIDENCE/cr-delete.txt"
kubectl delete clusterrolebinding "$delegator" --ignore-not-found \
  >> "$AUTH_EVIDENCE/cr-delete.txt"
for sa in auth-reader auth-writer auth-admin auth-operator auth-unbound auth-sibling; do
  kubectl -n "$AUTH_NS" delete serviceaccount "$sa" --ignore-not-found \
    >> "$AUTH_EVIDENCE/cr-delete.txt"
  kubectl -n "$AUTH_NS" delete role,rolebinding \
    -l "app.kubernetes.io/instance=$sa,app.kubernetes.io/component=access" \
    --ignore-not-found >> "$AUTH_EVIDENCE/cr-delete.txt"
done
kubectl delete namespace "$CLIENT_NS" --ignore-not-found --wait=false \
  >> "$AUTH_EVIDENCE/cr-delete.txt"
stop_forward
teardown=1

# R8/AC8 — nothing that could be replayed survives in the retained evidence.
# The canary is each credential's own high-entropy tail, so this checks the
# material rather than a pattern that resembles it.
echo ">> sweeping retained evidence for credential material"
leaked=0
for credential in "$SECRET_DIR"/*.token "$SECRET_DIR"/gsa-access-token \
  "$SECRET_DIR"/google-id-token "$SECRET_DIR"/human-access-token; do
  [[ -f "$credential" ]] || continue
  canary="$(tr -d '\n' < "$credential" | tail -c 32)"
  [[ ${#canary} -ge 16 ]] || continue
  if grep -rqF "$canary" "$EVIDENCE_DIR" 2>/dev/null; then
    echo "credential material from $(basename "$credential") appears in retained evidence" >&2
    leaked=1
  fi
done
if grep -rqE '"token"[[:space:]]*:' "$AUTH_EVIDENCE" 2>/dev/null; then
  echo "a retained auth evidence file carries a token field" >&2
  leaked=1
fi
(( leaked == 0 )) || exit 1
redaction=1

jq -n \
  --arg schema "axiom.gcp.lumen.auth.acceptance.v1" \
  --arg run_id "$RUN_ID" \
  --arg project_id "$PROJECT_ID" \
  --arg namespace "$AUTH_NS" \
  --arg instance "$INSTANCE" \
  --arg audience "$AUDIENCE" \
  --arg human_user "$HUMAN_USER" \
  --arg human_cluster_admin "$HUMAN_ADMIN" \
  --arg gsa_user "$GSA_USER" \
  --arg gsa_cluster_admin "$GSA_ADMIN" \
  --argjson sibling_refusals "$sibling_refusals" \
  --argjson identity_observations "$identity_observations" \
  --argjson non_ksa_rejections "$non_ksa_rejections" \
  --argjson authorization "$authorization" \
  --argjson revocations "$revocations" \
  --argjson redaction "$redaction" \
  --argjson teardown "$teardown" \
  --argjson issuer_revocation_seconds "$issuer_revoke_seconds" \
  --argjson lumen_revocation_seconds "$lumen_revoke_seconds" \
  --argjson revocation_bound_seconds "$REVOCATION_BOUND_SECONDS" \
  --arg verified_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  '{
     schema: $schema,
     run_id: $run_id,
     project_id: $project_id,
     namespace: $namespace,
     instance: $instance,
     audience: $audience,
     issuers: [
       {kind: "google-user", kubernetes_username: $human_user, cluster_admin: ($human_cluster_admin == "yes")},
       {kind: "google-service-account", kubernetes_username: $gsa_user, cluster_admin: ($gsa_cluster_admin == "yes")}
     ],
     sibling_mint_refusals: $sibling_refusals,
     rows: {
       identity_observations: $identity_observations,
       non_ksa_rejections: $non_ksa_rejections,
       authorization: $authorization,
       sibling_refusals: $sibling_refusals,
       revocations: $revocations,
       redaction: $redaction,
       teardown: $teardown
     },
     revocation: {
       issuer_token_request_seconds: $issuer_revocation_seconds,
       lumen_authorization_seconds: $lumen_revocation_seconds,
       documented_bound_seconds: $revocation_bound_seconds
     },
     status: "passed",
     verified_at: $verified_at
   }' > "$EVIDENCE_DIR/lumen-auth-acceptance.json"

# SPEC-MANAGED: apps/lumen/tech-design/src/lumen/work_items/wi_12_18_lumen_auth_phase_2_prove_two_hop_ksa_rbac_authorization_on.py
# HANDWRITE-BEGIN tracker="#2879"
lumen_auth_redaction_audit_and_destroy() {
  "${LUMEN_AUTH_REDACTION_AUDITOR:?required}" \
    --evidence-root "$EVIDENCE_DIR" \
    --credential-dir "$SECRET_DIR" \
    --output "${LUMEN_AUTH_REDACTION_AUDIT_PATH:?required}"
  rm -rf "$SECRET_DIR"
  SECRET_DIR=""
}
if [[ -n "${LUMEN_AUTH_REDACTION_AUDITOR:-}" || -n "${LUMEN_AUTH_REDACTION_AUDIT_PATH:-}" ]]; then
  [[ -n "${LUMEN_AUTH_REDACTION_AUDITOR:-}" && -n "${LUMEN_AUTH_REDACTION_AUDIT_PATH:-}" ]] || {
    echo "LUMEN_AUTH_REDACTION_AUDITOR and LUMEN_AUTH_REDACTION_AUDIT_PATH must be set together" >&2
    exit 1
  }
  lumen_auth_redaction_audit_and_destroy
fi
# HANDWRITE-END

echo ">> two-hop KSA/RBAC authorization proved on GKE"
