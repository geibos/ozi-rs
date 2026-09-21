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
# machine and is not used for releases — release signing is a separate concern.
#
# Run: ./scripts/setup-dev-signing.sh
# macOS asks for your login password once, when the certificate is marked
# trusted for code signing. The script is safe to re-run: each step is skipped
# when it is already done.
set -euo pipefail

CERT_NAME="ozi-rs Local Dev"
KEYCHAIN="$HOME/Library/Keychains/login.keychain-db"
CERT_DIR="$HOME/.config/ozi-rs"
CERT_PEM="$CERT_DIR/dev-signing-cert.pem"
KEY_PEM="$CERT_DIR/dev-signing-key.pem"

if security find-identity -v -p codesigning | grep -qF "$CERT_NAME"; then
  echo "Identity '$CERT_NAME' already exists — nothing to do."
  exit 0
fi

mkdir -p "$CERT_DIR"
chmod 700 "$CERT_DIR"

if [ ! -f "$CERT_PEM" ] || [ ! -f "$KEY_PEM" ]; then
  echo "==> Generating a self-signed code-signing certificate"
  # PKCS#12 is deliberately avoided: OpenSSL 3 defaults to AES-256 + PBKDF2,
  # which the macOS Security framework refuses to import ("MAC verification
  # failed"). Importing the key and the certificate separately sidesteps the
  # container format entirely.
  openssl req -x509 -newkey rsa:2048 -nodes -days 3650 \
    -keyout "$KEY_PEM" -out "$CERT_PEM" \
    -subj "/CN=$CERT_NAME" \
    -addext "basicConstraints=critical,CA:false" \
    -addext "keyUsage=critical,digitalSignature" \
    -addext "extendedKeyUsage=critical,codeSigning" >/dev/null 2>&1
  chmod 600 "$KEY_PEM"
else
  echo "==> Reusing the certificate already in $CERT_DIR"
fi

if ! security find-certificate -c "$CERT_NAME" "$KEYCHAIN" >/dev/null 2>&1; then
  echo "==> Importing the private key and certificate into the login keychain"
  # -T /usr/bin/codesign lets codesign use the key without a prompt per build.
  security import "$KEY_PEM" -k "$KEYCHAIN" -T /usr/bin/codesign >/dev/null
  security import "$CERT_PEM" -k "$KEYCHAIN" -T /usr/bin/codesign >/dev/null
else
  echo "==> Certificate already in the keychain"
fi

echo "==> Marking it trusted for code signing (macOS will ask for your password)"
security add-trusted-cert -d -r trustRoot -p codeSign -k "$KEYCHAIN" "$CERT_PEM"

# Without this, codesign prompts for keychain access on every signing run.
security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "" "$KEYCHAIN" >/dev/null 2>&1 || true

if security find-identity -v -p codesigning | grep -qF "$CERT_NAME"; then
  echo
  echo "Done. '$CERT_NAME' is available for code signing."
  echo "Builds sign with it automatically (the 'sign-dev' recipe runs after 'just build'),"
  echo "so the Documents-access prompt should appear once and then stay answered."
else
  echo "The identity was not created. Check the output above." >&2
  exit 1
fi
