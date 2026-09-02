#!/usr/bin/env bash
set -euo pipefail

: "${ACCEPTANCE_APPS:=lumen sift}"
: "${RUN_ID:?RUN_ID is required}"
: "${MANIFEST_DIR:?MANIFEST_DIR is required}"
: "${GKE_CLUSTER_NAME:?GKE_CLUSTER_NAME is required}"
: "${GKE_ZONE:?GKE_ZONE is required}"
: "${PROJECT_ID:?PROJECT_ID is required}"

# Determine acceptance mode and require only the appropriate CLIs/images
case "$ACCEPTANCE_APPS" in
  "lumen sift"|"lumen auth")
    : "${LUMEN_CLI:?LUMEN_CLI is required for lumen-sift mode}"
    : "${LUMEN_IMAGE:?LUMEN_IMAGE digest reference is required for lumen-sift mode}"
    [[ -x "$LUMEN_CLI" ]] || {
      echo "deployment CLI is not executable: $LUMEN_CLI" >&2
      exit 1
    }
    if [[ "$ACCEPTANCE_APPS" == "lumen sift" ]]; then
    : "${SIFT_CLI:?SIFT_CLI is required for lumen-sift mode}"
    : "${SIFT_IMAGE:?SIFT_IMAGE digest reference is required for lumen-sift mode}"
    [[ -x "$SIFT_CLI" ]] || {
      echo "deployment CLI is not executable: $SIFT_CLI" >&2
      exit 1
    }
    fi
    mkdir -p "$MANIFEST_DIR/lumen/operator" "$MANIFEST_DIR/lumen/instance"
    if [[ "$ACCEPTANCE_APPS" == "lumen sift" ]]; then
      mkdir -p "$MANIFEST_DIR/sift/operator" "$MANIFEST_DIR/sift/instance" "$MANIFEST_DIR/sift/collector"
    fi
    ;;
  "sift")
    : "${SIFT_CLI:?SIFT_CLI is required for sift mode}"
    : "${SIFT_IMAGE:?SIFT_IMAGE digest reference is required for sift mode}"
    : "${RIG_IMAGE:?RIG_IMAGE digest reference is required for sift mode}"
    [[ -x "$SIFT_CLI" ]] || {
      echo "deployment CLI is not executable: $SIFT_CLI" >&2
      exit 1
    }
    mkdir -p "$MANIFEST_DIR/sift/operator" "$MANIFEST_DIR/sift/instance"
    ;;
  "tape")
    : "${TAPE_CLI:?TAPE_CLI is required for tape mode}"
    : "${TAPE_IMAGE:?TAPE_IMAGE digest reference is required for tape mode}"
    [[ -x "$TAPE_CLI" ]] || {
      echo "deployment CLI is not executable: $TAPE_CLI" >&2
      exit 1
    }
    mkdir -p \
      "$MANIFEST_DIR/tape/operator" \
      "$MANIFEST_DIR/tape/instance"
    ;;
  *)
    echo "ACCEPTANCE_APPS must be 'lumen sift', 'lumen auth', 'sift', or 'tape'" >&2
    exit 1
    ;;
esac

# Render Lumen manifests for both Lumen modes.
if [[ "$ACCEPTANCE_APPS" == "lumen sift" || "$ACCEPTANCE_APPS" == "lumen auth" ]]; then
  "$LUMEN_CLI" k8s crd render --out "$MANIFEST_DIR/lumen/crd.yaml"
  "$LUMEN_CLI" k8s operator render --namespace lumen-system \
    --out "$MANIFEST_DIR/lumen/operator/operator.yaml"
  "$LUMEN_CLI" k8s instance render --profile dev --name lumen --namespace lumen \
    --image "$LUMEN_IMAGE" --out "$MANIFEST_DIR/lumen/instance/lumen.yaml"

fi

if [[ "$ACCEPTANCE_APPS" == "lumen sift" || "$ACCEPTANCE_APPS" == "sift" ]]; then
  "$SIFT_CLI" k8s crd render --out "$MANIFEST_DIR/sift/crd.yaml"
  "$SIFT_CLI" k8s operator render --namespace sift-system \
    --out "$MANIFEST_DIR/sift/operator/operator.yaml"
  "$SIFT_CLI" k8s instance render --profile dev \
    --out "$MANIFEST_DIR/sift/instance/sift.yaml"
  if [[ "$ACCEPTANCE_APPS" == "lumen sift" ]]; then
    mkdir -p "$MANIFEST_DIR/sift/collector"
    "$SIFT_CLI" k8s collector render --namespace sift --image "$SIFT_IMAGE" \
      --out "$MANIFEST_DIR/sift/collector/collector.yaml"
  fi
fi

# Render Tape manifests for tape mode
if [[ "$ACCEPTANCE_APPS" == "tape" ]]; then
  "$TAPE_CLI" k8s crd render --out "$MANIFEST_DIR/tape/crd.yaml"
  "$TAPE_CLI" k8s operator render --namespace tape-system \
    --out "$MANIFEST_DIR/tape/operator/operator.yaml"
  "$TAPE_CLI" k8s instance render --profile dev --name tape --namespace tape \
    --image "$TAPE_IMAGE" --out "$MANIFEST_DIR/tape/instance/tape.yaml"
fi

if [[ "$ACCEPTANCE_APPS" == "lumen sift" || "$ACCEPTANCE_APPS" == "lumen auth" ]]; then
cat > "$MANIFEST_DIR/lumen/operator/kustomization.yaml" <<EOF
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - operator.yaml
patches:
  - target:
      group: apps
      version: v1
      kind: Deployment
      name: lumen-operator
    patch: |-
      - op: replace
        path: /spec/template/spec/containers/0/image
        value: ${LUMEN_IMAGE}
EOF
fi

if [[ "$ACCEPTANCE_APPS" == "lumen sift" || "$ACCEPTANCE_APPS" == "lumen auth" ]]; then
  if [[ "$ACCEPTANCE_APPS" == "lumen sift" ]]; then
    : "${BACKUP_BUCKET:?BACKUP_BUCKET is required}"
    : "${BACKUP_GSA_EMAIL:?BACKUP_GSA_EMAIL is required}"
cat > "$MANIFEST_DIR/lumen/instance/identity.yaml" <<EOF
apiVersion: v1
kind: Namespace
metadata:
  name: lumen
---
apiVersion: v1
kind: ServiceAccount
metadata:
  name: lumen-backup
  namespace: lumen
  annotations:
    iam.gke.io/gcp-service-account: ${BACKUP_GSA_EMAIL}
EOF
  else
cat > "$MANIFEST_DIR/lumen/instance/identity.yaml" <<EOF
apiVersion: v1
kind: Namespace
metadata:
  name: lumen
EOF
  fi
  instance_resources="identity.yaml"

cat > "$MANIFEST_DIR/lumen/instance/kustomization.yaml" <<EOF
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  ${instance_resources:+- $instance_resources}
  - lumen.yaml
patches:
  - target:
      group: lumen.dev
      version: v1alpha1
      kind: Lumen
      name: lumen
    patch: |-
      - op: add
        path: /spec/imagePullPolicy
        value: IfNotPresent
      - op: replace
        path: /spec/serving/cpu
        value: 500m
      - op: replace
        path: /spec/serving/memory
        value: 1Gi
      - op: replace
        path: /spec/logFormat
        value: json
      - op: add
        path: /spec/serving/raftStorage
        value: 1Gi
$(if [[ "$ACCEPTANCE_APPS" == "lumen sift" ]]; then cat <<PATCH
      - op: add
        path: /spec/serving/backup
        value:
          schedule: "*/5 * * * *"
          destination: gs://${BACKUP_BUCKET}/lumen/${RUN_ID}
          retentionSecs: 3600
PATCH
fi)
EOF

kubectl kustomize "$MANIFEST_DIR/lumen/operator" > "$MANIFEST_DIR/lumen/operator.bundle.yaml"
kubectl kustomize "$MANIFEST_DIR/lumen/instance" > "$MANIFEST_DIR/lumen/instance.bundle.yaml"

fi

if [[ "$ACCEPTANCE_APPS" == "lumen sift" || "$ACCEPTANCE_APPS" == "sift" ]]; then
  : "${BACKUP_BUCKET:?BACKUP_BUCKET is required}"
  : "${BACKUP_GSA_EMAIL:?BACKUP_GSA_EMAIL is required}"
  peer_dir="$MANIFEST_DIR/sift/peer-pki"
  mkdir -p "$peer_dir"
  cat > "$peer_dir/leaf.ext" <<'EOF'
basicConstraints=critical,CA:FALSE
keyUsage=critical,digitalSignature,keyEncipherment
extendedKeyUsage=serverAuth,clientAuth
subjectAltName=DNS:*.sift-store-headless.sift.svc.cluster.local,DNS:*.sift-control-headless.sift.svc.cluster.local,DNS:sift-store.sift.svc.cluster.local,DNS:sift-control.sift.svc.cluster.local,DNS:*.sift-restore-store-headless.sift-restore.svc.cluster.local,DNS:*.sift-restore-control-headless.sift-restore.svc.cluster.local,DNS:sift-restore-store.sift-restore.svc.cluster.local,DNS:sift-restore-control.sift-restore.svc.cluster.local
EOF
  openssl req -x509 -newkey rsa:2048 -nodes -days 2 \
    -subj "/CN=Sift MVP peer CA ${RUN_ID}" \
    -keyout "$peer_dir/ca.key" -out "$peer_dir/ca.crt" >/dev/null 2>&1
  openssl req -newkey rsa:2048 -nodes -subj "/CN=sift-peer" \
    -keyout "$peer_dir/tls.key" -out "$peer_dir/tls.csr" >/dev/null 2>&1
  openssl x509 -req -days 2 -sha256 -in "$peer_dir/tls.csr" \
    -CA "$peer_dir/ca.crt" -CAkey "$peer_dir/ca.key" -CAcreateserial \
    -extfile "$peer_dir/leaf.ext" -out "$peer_dir/tls.crt" >/dev/null 2>&1

cat > "$MANIFEST_DIR/sift/operator/kustomization.yaml" <<EOF
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - operator.yaml
patches:
  - target:
      group: apps
      version: v1
      kind: Deployment
      name: sift-operator
    patch: |-
      - op: replace
        path: /spec/template/spec/containers/0/image
        value: ${SIFT_IMAGE}
EOF

cat > "$MANIFEST_DIR/sift/instance/identity.yaml" <<EOF
apiVersion: v1
kind: Namespace
metadata:
  name: sift
---
apiVersion: v1
kind: ServiceAccount
metadata:
  name: sift-store
  namespace: sift
  annotations:
    iam.gke.io/gcp-service-account: ${BACKUP_GSA_EMAIL}
---
apiVersion: v1
kind: ServiceAccount
metadata:
  name: sift-rig
  namespace: sift
---
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: sift-rig-project
  namespace: sift
rules:
  - apiGroups: ["sift.axiom.dev"]
    resources: ["projects"]
    resourceNames: ["sift-mvp", "sift-mvp-alt"]
    verbs: ["get", "create", "update"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: sift-rig-project
  namespace: sift
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: Role
  name: sift-rig-project
subjects:
  - kind: ServiceAccount
    name: sift-rig
    namespace: sift
EOF

printf '%s\n' '---' >> "$MANIFEST_DIR/sift/instance/identity.yaml"
kubectl create secret generic sift-peer-tls --namespace sift \
  --from-file=tls.crt="$peer_dir/tls.crt" \
  --from-file=tls.key="$peer_dir/tls.key" \
  --from-file=ca.crt="$peer_dir/ca.crt" \
  --dry-run=client -o yaml >> "$MANIFEST_DIR/sift/instance/identity.yaml"

if [[ "$ACCEPTANCE_APPS" == "sift" ]]; then
  sift_store_size=50Gi
  sift_control_size=5Gi
  sift_gateway_size=2Gi
  sift_query_size=2Gi
  sift_auth=kubernetes
else
  sift_store_size=1Gi
  sift_control_size=1Gi
  sift_gateway_size=1Gi
  sift_query_size=1Gi
  sift_auth=off
fi

cat > "$MANIFEST_DIR/sift/instance/kustomization.yaml" <<EOF
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - identity.yaml
  - sift.yaml
patches:
  - target:
      group: sift.axiom.dev
      version: v1alpha1
      kind: Sift
      name: sift
    patch: |-
      - op: replace
        path: /metadata/namespace
        value: sift
      - op: replace
        path: /spec/image
        value: ${SIFT_IMAGE}
      - op: replace
        path: /spec/peerTlsSecret
        value: sift-peer-tls
      - op: replace
        path: /spec/auth
        value: ${sift_auth}
      - op: replace
        path: /spec/storage/storeSize
        value: ${sift_store_size}
      - op: replace
        path: /spec/storage/controlSize
        value: ${sift_control_size}
      - op: replace
        path: /spec/storage/gatewaySize
        value: ${sift_gateway_size}
      - op: replace
        path: /spec/storage/querySize
        value: ${sift_query_size}
      - op: add
        path: /spec/archive
        value:
          destination: gs://${BACKUP_BUCKET}/sift/${RUN_ID}
      - op: replace
        path: /spec/gcpProjectId
        value: ${PROJECT_ID}
      - op: replace
        path: /spec/gkeClusterName
        value: ${GKE_CLUSTER_NAME}
      - op: replace
        path: /spec/gkeLocation
        value: ${GKE_ZONE}
$(if [[ "$ACCEPTANCE_APPS" == "sift" ]]; then cat <<PATCH
      - op: add
        path: /spec/placement
        value:
          nodeSelector:
            axiom-run-id: ${RUN_ID}
PATCH
fi)
EOF

kubectl kustomize "$MANIFEST_DIR/sift/operator" > "$MANIFEST_DIR/sift/operator.bundle.yaml"
kubectl kustomize "$MANIFEST_DIR/sift/instance" > "$MANIFEST_DIR/sift/instance.bundle.yaml"

if [[ "$ACCEPTANCE_APPS" == "lumen sift" ]]; then
cat > "$MANIFEST_DIR/sift/collector/config.yaml" <<EOF
apiVersion: v1
kind: ConfigMap
metadata:
  name: sift-collector
  namespace: sift
data:
  endpoint: http://sift.sift.svc.cluster.local:7380
  project: operator-acceptance
  environment: gke
  gcpProjectId: ${PROJECT_ID}
  clusterName: ${GKE_CLUSTER_NAME}
  location: ${GKE_ZONE}
---
apiVersion: v1
kind: Secret
metadata:
  name: sift-collector
  namespace: sift
type: Opaque
stringData:
  token: ""
EOF

cat > "$MANIFEST_DIR/sift/collector/kustomization.yaml" <<EOF
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - config.yaml
  - collector.yaml
EOF

kubectl kustomize "$MANIFEST_DIR/sift/collector" > "$MANIFEST_DIR/sift/collector.bundle.yaml"
fi
fi

# Tape-specific kustomization for tape mode
if [[ "$ACCEPTANCE_APPS" == "tape" ]]; then
cat > "$MANIFEST_DIR/tape/operator/kustomization.yaml" <<EOF
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - operator.yaml
patches:
  - target:
      group: apps
      version: v1
      kind: Deployment
      name: tape-operator
    patch: |-
      - op: replace
        path: /spec/template/spec/containers/0/image
        value: ${TAPE_IMAGE}
EOF

cat > "$MANIFEST_DIR/tape/instance/identity.yaml" <<EOF
apiVersion: v1
kind: Namespace
metadata:
  name: tape
---
apiVersion: v1
kind: ServiceAccount
metadata:
  name: tape-backup
  namespace: tape
  annotations:
    iam.gke.io/gcp-service-account: ${BACKUP_GSA_EMAIL}
EOF

# The Tape CRD DOES now carry a CR-native `backup` field (#2574), matching
# Lumen/Sift — the earlier note here ("run 0724154839: a /spec/backup patch is
# rejected by strict decoding") described the pre-#2574 CRD and is no longer
# true. The `tape-backup` ServiceAccount below is now also rendered by the
# operator; it stays here because this copy carries the Workload Identity
# annotation, which survives reconcile (a different field manager owns it).
#
# The CronJob itself is still hand-rolled, for one reason: the shared
# `service_k8s::render::cron_job` helper has no `suspend` knob, so a
# CR-native CronJob would be UNSUSPENDED. An unsuspended `*/5` schedule kept
# firing against the torn-down 1x1 instance during the cold-restore rebuild
# (run 0723080156) — see the `suspend: true` note below. Switching this to
# `/spec/backup` needs that knob first and a GKE run to re-prove the
# cold-restore phase; it is deliberately not a drive-by edit.
#
# ENTRYPOINT is already the `tape` binary (images/Dockerfile.tape), so
# `args` alone selects the subcommand — no `command` override.
cat > "$MANIFEST_DIR/tape/instance/backup-cronjob.yaml" <<EOF
apiVersion: batch/v1
kind: CronJob
metadata:
  name: tape-backup
  namespace: tape
spec:
  schedule: "*/5 * * * *"
  # The acceptance flow triggers exactly one Job via
  # \`kubectl create job --from=cronjob/tape-backup\` (which works on a
  # suspended CronJob). An unsuspended schedule kept firing against the
  # torn-down 1x1 instance during the cold-restore rebuild (run 0723080156).
  suspend: true
  concurrencyPolicy: Forbid
  jobTemplate:
    spec:
      template:
        spec:
          serviceAccountName: tape-backup
          restartPolicy: Never
          containers:
            - name: backup
              image: ${TAPE_IMAGE}
              args:
                - backup
                - --url
                - http://tape.tape.svc.cluster.local:7137
                - --dest
                - gs://${BACKUP_BUCKET}/tape/${RUN_ID}
                - --retention-secs
                - "3600"
EOF

cat > "$MANIFEST_DIR/tape/instance/kustomization.yaml" <<EOF
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - identity.yaml
  - tape.yaml
  - backup-cronjob.yaml
patches:
  - target:
      group: tape.dev
      version: v1alpha1
      kind: Tape
      name: tape
    patch: |-
      - op: add
        path: /spec/imagePullPolicy
        value: IfNotPresent
      - op: replace
        path: /spec/resources/cpu
        value: 500m
      - op: replace
        path: /spec/resources/memory
        value: 1Gi
      - op: replace
        path: /spec/storage
        value: 1Gi
      - op: add
        path: /spec/topics
        value:
          - name: acceptance
            subscriptions:
              - acceptance-sub
EOF

kubectl kustomize "$MANIFEST_DIR/tape/operator" > "$MANIFEST_DIR/tape/operator.bundle.yaml"
kubectl kustomize "$MANIFEST_DIR/tape/instance" > "$MANIFEST_DIR/tape/instance.bundle.yaml"
fi
