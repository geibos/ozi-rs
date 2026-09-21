#!/usr/bin/env bash
# Create a local code-signing identity for development builds.
#
# Why: a debug bundle is ad-hoc signed, and its code directory hash changes with
# every build. macOS treats each rebuild as a different program, so the
# "ozi-rs would like to access files in your Documents folder" prompt comes back
# on every single build and the app will not open its window until it is
# answered. That makes screenshot and smoke verification impossible to automate.
#
# A stable self-signed certificate fixes this: the app keeps one identity across
# rebuilds, so the grant is given once.
#
# This is a one-time, local-only setup. The certificate never leaves this
# machine, is not a notarisation identity, and is not used for releases —
# release signing is a separate, later concern.
#
# Run: ./scripts/setup-dev-signing.sh
# macOS will ask for your login password when the certificate is marked trusted.
set -euo pipefail

CERT_NAME="ozi-rs Local Dev"
KEYCHAIN="$HOME/Library/Keychains/login.keychain-db"
WORK_DIR="$(mktemp -d)"
trap 'rm -rf "$WORK_DIR"' EXIT

if security find-identity -v -p codesigning | grep -qF "$CERT_NAME"; then
  echo "Identity '$CERT_NAME' already exists — nothing to do."
  exit 0
fi

echo "==> Generating a self-signed code-signing certificate"
openssl req -x509 -newkey rsa:2048 -nodes -days 3650 \
  -keyout "$WORK_DIR/key.pem" -out "$WORK_DIR/cert.pem" \
  -subj "/CN=$CERT_NAME" \
  -addext "basicConstraints=critical,CA:false" \
  -addext "keyUsage=critical,digitalSignature" \
  -addext "extendedKeyUsage=critical,codeSigning" >/dev/null 2>&1

openssl pkcs12 -export -out "$WORK_DIR/cert.p12" \
  -inkey "$WORK_DIR/key.pem" -in "$WORK_DIR/cert.pem" \
  -passout pass: >/dev/null 2>&1

echo "==> Importing it into the login keychain"
# -T /usr/bin/codesign lets codesign use the key without a prompt per build.
security import "$WORK_DIR/cert.p12" -k "$KEYCHAIN" -P "" -T /usr/bin/codesign >/dev/null

echo "==> Marking it trusted for code signing (macOS will ask for your password)"
security add-trusted-cert -d -r trustRoot -p codeSign -k "$KEYCHAIN" "$WORK_DIR/cert.pem"

# Without this, codesign prompts for keychain access on every signing run.
security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "" "$KEYCHAIN" >/dev/null 2>&1 || true

if security find-identity -v -p codesigning | grep -qF "$CERT_NAME"; then
  echo
  echo "Done. '$CERT_NAME' is available for code signing."
  echo "Builds now sign with it automatically (see the 'sign-dev' recipe in the justfile),"
  echo "so the Documents-access prompt should appear once and then stay answered."
else
  echo "The identity was not created. Check the output above." >&2
  exit 1
fi
