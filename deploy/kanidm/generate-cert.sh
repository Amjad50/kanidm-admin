#!/bin/sh
# Generate a self-signed cert for kanidm's internal TLS port, if not
# already present. This is the cert kanidm uses on :8443 inside the
# docker network. Caddy validates it via the SAN entries and we
# explicitly mark it "accept-invalid-certs" in admin-ui's config (it's
# pinned via the SAN + the internal hostname, not a public CA chain).
set -eu

CERT_DIR="${CERT_DIR:-/certs}"
mkdir -p "$CERT_DIR"

if [ -f "$CERT_DIR/cert.pem" ] && [ -f "$CERT_DIR/key.pem" ]; then
    echo "kanidm cert already exists at $CERT_DIR — leaving alone"
    exit 0
fi

echo "generating self-signed cert for kanidm (CN=idm.localhost, SAN=kanidm,idm.localhost,localhost)"

openssl req -x509 -newkey rsa:4096 \
    -keyout "$CERT_DIR/key.pem" \
    -out "$CERT_DIR/cert.pem" \
    -days 3650 -nodes \
    -subj "/CN=idm.localhost" \
    -addext "subjectAltName=DNS:idm.localhost,DNS:kanidm,DNS:localhost,IP:127.0.0.1"

# kanidm runs as uid 389 inside the official image.
chown -R 389:389 "$CERT_DIR" 2>/dev/null || true
chmod 0640 "$CERT_DIR/key.pem"
chmod 0644 "$CERT_DIR/cert.pem"

echo "cert generated. Valid for 10 years."
