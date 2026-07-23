#!/usr/bin/env bash
set -euo pipefail

: "${LUMEN_CLI:?LUMEN_CLI is required}"
: "${LUMEN_IMAGE:?LUMEN_IMAGE digest reference is required}"
: "${LUMEN_ONLY:=0}"
: "${BACKUP_BUCKET:?BACKUP_BUCKET is required}"
: "${BACKUP_GSA_EMAIL:?BACKUP_GSA_EMAIL is required}"
: "${RUN_ID:?RUN_ID is required}"
: "${MANIFEST_DIR:?MANIFEST_DIR is required}"
: "${GKE_CLUSTER_NAME:?GKE_CLUSTER_NAME is required}"
: "${GKE_ZONE:?GKE_ZONE is required}"
: "${PROJECT_ID:?PROJECT_ID is required}"

for cli in "$LUMEN_CLI"; do
  [[ -x "$cli" ]] || {
    echo "deployment CLI is not executable: $cli" >&2
    exit 1
  }
done
if [[ "$LUMEN_ONLY" != "1" ]]; then
  : "${SIFT_CLI:?SIFT_CLI is required outside LUMEN_ONLY mode}"
  : "${SIFT_IMAGE:?SIFT_IMAGE digest reference is required outside LUMEN_ONLY mode}"
  [[ -x "$SIFT_CLI" ]] || {
    echo "deployment CLI is not executable: $SIFT_CLI" >&2
    exit 1
  }
fi

mkdir -p \
  "$MANIFEST_DIR/lumen/operator" \
  "$MANIFEST_DIR/lumen/instance"
if [[ "$LUMEN_ONLY" != "1" ]]; then
  mkdir -p \
    "$MANIFEST_DIR/sift/operator" \
    "$MANIFEST_DIR/sift/instance" \
    "$MANIFEST_DIR/sift/collector"
fi

"$LUMEN_CLI" k8s crd render --out "$MANIFEST_DIR/lumen/crd.yaml"
"$LUMEN_CLI" k8s operator render --namespace lumen-system \
  --out "$MANIFEST_DIR/lumen/operator/operator.yaml"
"$LUMEN_CLI" k8s instance render --profile dev --name lumen --namespace lumen \
  --image "$LUMEN_IMAGE" --out "$MANIFEST_DIR/lumen/instance/lumen.yaml"

if [[ "$LUMEN_ONLY" != "1" ]]; then
  "$SIFT_CLI" k8s crd render --out "$MANIFEST_DIR/sift/crd.yaml"
  "$SIFT_CLI" k8s operator render --namespace sift-system \
    --out "$MANIFEST_DIR/sift/operator/operator.yaml"
  "$SIFT_CLI" k8s instance render --profile dev \
    --out "$MANIFEST_DIR/sift/instance/sift.yaml"
  "$SIFT_CLI" k8s collector render --namespace sift --image "$SIFT_IMAGE" \
    --out "$MANIFEST_DIR/sift/collector/collector.yaml"
fi

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

cat > "$MANIFEST_DIR/lumen/instance/kustomization.yaml" <<EOF
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - identity.yaml
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
      - op: add
        path: /spec/serving/backup
        value:
          schedule: "*/5 * * * *"
          destination: gs://${BACKUP_BUCKET}/lumen/${RUN_ID}
          retentionSecs: 3600
EOF

kubectl kustomize "$MANIFEST_DIR/lumen/operator" > "$MANIFEST_DIR/lumen/operator.bundle.yaml"
kubectl kustomize "$MANIFEST_DIR/lumen/instance" > "$MANIFEST_DIR/lumen/instance.bundle.yaml"

if [[ "$LUMEN_ONLY" != "1" ]]; then
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
  name: sift-backup
  namespace: sift
  annotations:
    iam.gke.io/gcp-service-account: ${BACKUP_GSA_EMAIL}
EOF

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
        path: /spec/dataSize
        value: 1Gi
      - op: add
        path: /spec/backup
        value:
          schedule: "*/5 * * * *"
          destination: gs://${BACKUP_BUCKET}/sift/${RUN_ID}
          retentionSecs: 3600
EOF

kubectl kustomize "$MANIFEST_DIR/sift/operator" > "$MANIFEST_DIR/sift/operator.bundle.yaml"
kubectl kustomize "$MANIFEST_DIR/sift/instance" > "$MANIFEST_DIR/sift/instance.bundle.yaml"

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
