#!/bin/bash
# Create Kubernetes TLS secret for ISSUN relay server

set -e

NAMESPACE=${NAMESPACE:-default}
SECRET_NAME="issun-tls-certs"

echo "Creating TLS certificates for ISSUN relay server..."

# Check if certificates already exist
if [ ! -f "../../certs/cert.pem" ] || [ ! -f "../../certs/key.pem" ]; then
    echo "Certificates not found. Generating self-signed certificates..."
    mkdir -p ../../certs
    cd ../../certs
    openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem \
        -days 365 -nodes -subj "/CN=issun-relay"
    cd -
    echo "✅ Certificates generated in ../../certs/"
fi

echo "Creating Kubernetes secret in namespace: $NAMESPACE"

kubectl create secret generic $SECRET_NAME \
    --from-file=tls.crt=../../certs/cert.pem \
    --from-file=tls.key=../../certs/key.pem \
    --namespace=$NAMESPACE \
    --dry-run=client -o yaml | kubectl apply -f -

echo "✅ Secret '$SECRET_NAME' created in namespace '$NAMESPACE'"
echo ""
echo "You can now deploy the relay server:"
echo "  kubectl apply -f deployment.yaml"
