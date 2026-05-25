#!/usr/bin/env bash
# Create a read-only Firestore service account for the CV pipeline.
# Output: firebase-seed/out/cv-reader-key.json (gitignored)
#
# Prerequisites:
#   gcloud auth login
#   gcloud config set project battleplan-dev-2024

set -euo pipefail

PROJECT="${FIRESTORE_PROJECT:-battleplan-dev-2024}"
SA_NAME="cv-reader"
SA_EMAIL="${SA_NAME}@${PROJECT}.iam.gserviceaccount.com"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT_DIR="${SCRIPT_DIR}/../../firebase-seed/out"
KEY_PATH="${OUT_DIR}/cv-reader-key.json"

mkdir -p "${OUT_DIR}"

echo "Project: ${PROJECT}"
echo "Service account: ${SA_EMAIL}"

if ! gcloud iam service-accounts describe "${SA_EMAIL}" --project "${PROJECT}" &>/dev/null; then
  echo "Creating service account..."
  gcloud iam service-accounts create "${SA_NAME}" \
    --project "${PROJECT}" \
    --display-name="CV pipeline Firestore reader"
else
  echo "Service account already exists."
fi

echo "Granting roles/datastore.viewer..."
gcloud projects add-iam-policy-binding "${PROJECT}" \
  --member="serviceAccount:${SA_EMAIL}" \
  --role="roles/datastore.viewer" \
  --quiet >/dev/null

if [[ -f "${KEY_PATH}" ]]; then
  echo "Key file already exists — skipping: ${KEY_PATH}"
else
  echo "Creating key -> ${KEY_PATH}"
  gcloud iam service-accounts keys create "${KEY_PATH}" \
    --iam-account="${SA_EMAIL}" \
    --project="${PROJECT}"
fi

echo ""
echo "Next steps:"
echo "  1. Merge key JSON into raw_history_plain.json → firebase_credentials"
echo "  2. Re-run: CV_SECRET=... cargo run --bin encrypt"
echo "  3. Never commit ${KEY_PATH}"
