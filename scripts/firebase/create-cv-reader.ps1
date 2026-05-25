# Create a read-only Firestore service account for the CV pipeline.
# Output: firebase-seed/out/cv-reader-key.json (gitignored)
#
# Prerequisites:
#   gcloud auth login
#   gcloud config set project battleplan-dev-2024

$ErrorActionPreference = "Stop"

$Project = if ($env:FIRESTORE_PROJECT) { $env:FIRESTORE_PROJECT } else { "battleplan-dev-2024" }
$SaName = "cv-reader"
$SaEmail = "$SaName@$Project.iam.gserviceaccount.com"
$OutDir = Join-Path $PSScriptRoot "..\..\firebase-seed\out"
$KeyPath = Join-Path $OutDir "cv-reader-key.json"

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

Write-Host "Project: $Project"
Write-Host "Service account: $SaEmail"

$exists = gcloud iam service-accounts list --project $Project `
    --filter="email:$SaEmail" --format="value(email)" 2>$null

if (-not $exists) {
    Write-Host "Creating service account..."
    gcloud iam service-accounts create $SaName `
        --project $Project `
        --display-name "CV pipeline Firestore reader"
} else {
    Write-Host "Service account already exists."
}

Write-Host "Granting roles/datastore.viewer..."
gcloud projects add-iam-policy-binding $Project `
    --member "serviceAccount:$SaEmail" `
    --role "roles/datastore.viewer" `
    --condition=None `
    --quiet | Out-Null

if (Test-Path $KeyPath) {
    Write-Host "Key file already exists - skipping: $KeyPath"
    Write-Host "Delete it first if you need a new key."
} else {
    Write-Host "Creating key -> $KeyPath"
    gcloud iam service-accounts keys create $KeyPath `
        --iam-account $SaEmail `
        --project $Project
}

Write-Host ""
Write-Host "Next steps:"
Write-Host "  1. Merge key JSON into raw_history_plain.json -> firebase_credentials"
Write-Host "  2. Re-run: cargo run --bin encrypt"
Write-Host ('  3. Never commit ' + $KeyPath)
