# ISSUN Relay Server Deployment Guide

This directory contains deployment configurations for running the ISSUN relay server in various environments.

## Quick Start

### Docker Compose (Recommended for Development)

1. **Generate TLS certificates** (first time only):
   ```bash
   make certs
   ```

2. **Start the server**:
   ```bash
   docker-compose up -d
   ```

3. **Check logs**:
   ```bash
   docker-compose logs -f relay-server
   ```

4. **Stop the server**:
   ```bash
   docker-compose down
   ```

### Docker (Manual)

1. **Build the image**:
   ```bash
   docker build -f crates/issun-server/Dockerfile -t issun-relay-server:latest .
   ```

2. **Run the container**:
   ```bash
   docker run -d \
     --name issun-relay \
     -p 5000:5000/udp \
     -v $(pwd)/certs:/app/certs:ro \
     -e RUST_LOG=issun_server=info \
     issun-relay-server:latest
   ```

### Kubernetes

1. **Create TLS secret**:
   ```bash
   cd deploy/kubernetes
   ./create-tls-secret.sh
   ```

2. **Deploy the server**:
   ```bash
   kubectl apply -f deployment.yaml
   ```

3. **Check status**:
   ```bash
   kubectl get pods -l app=issun-relay
   kubectl logs -l app=issun-relay -f
   ```

4. **Get external IP** (for LoadBalancer):
   ```bash
   kubectl get service issun-relay-service
   ```

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ISSUN_BIND_ADDR` | `0.0.0.0:5000` | Server bind address |
| `ISSUN_CERT_PATH` | `/app/certs/cert.pem` | TLS certificate path |
| `ISSUN_KEY_PATH` | `/app/certs/key.pem` | TLS private key path |
| `ISSUN_MAX_CLIENTS` | `1000` | Maximum concurrent clients |
| `ISSUN_HEARTBEAT_INTERVAL` | `5` | Heartbeat interval (seconds) |
| `RUST_LOG` | `issun_server=info` | Logging level |

### TLS Certificates

#### Development (Self-signed)

Generated automatically with `make certs`:

```bash
openssl req -x509 -newkey rsa:4096 -keyout certs/key.pem -out certs/cert.pem \
  -days 365 -nodes -subj "/CN=localhost"
```

#### Production (Let's Encrypt)

For production deployments, use Let's Encrypt with cert-manager (Kubernetes) or Certbot:

```bash
certbot certonly --standalone -d relay.yourgame.com
```

Then mount the certificates:
- Certificate: `/etc/letsencrypt/live/relay.yourgame.com/fullchain.pem`
- Private key: `/etc/letsencrypt/live/relay.yourgame.com/privkey.pem`

## Cloud Deployments

### AWS ECS

See `aws/` directory for CloudFormation templates and task definitions.

### Google Cloud Run

Cloud Run doesn't support UDP (QUIC), so use GKE instead:

```bash
gcloud container clusters create issun-cluster \
  --zone us-central1-a \
  --num-nodes 2

kubectl apply -f kubernetes/deployment.yaml
```

### Fly.io

Fly.io supports UDP apps. Create `fly.toml`:

```toml
app = "issun-relay"

[build]
  dockerfile = "crates/issun-server/Dockerfile"

[[services]]
  internal_port = 5000
  protocol = "udp"

  [[services.ports]]
    port = 5000
```

Deploy:
```bash
fly deploy
```

### DigitalOcean App Platform

App Platform doesn't support UDP. Use a Droplet with Docker instead.

## Monitoring

### Health Checks

The server includes a basic health check that verifies the process is running:

```bash
# Docker
docker exec issun-relay pgrep -x issun-server

# Kubernetes
kubectl exec -it <pod-name> -- pgrep -x issun-server
```

### Logs

```bash
# Docker Compose
docker-compose logs -f relay-server

# Kubernetes
kubectl logs -l app=issun-relay -f

# Docker
docker logs -f issun-relay
```

### Metrics (Future)

Future versions will expose Prometheus metrics at `/metrics` endpoint.

## Scaling

### Horizontal Scaling

The relay server is stateless and can be scaled horizontally:

**Kubernetes:**
```bash
kubectl scale deployment issun-relay-server --replicas=3
```

**Docker Swarm:**
```bash
docker service scale issun-relay=3
```

### Load Balancing

Use a UDP load balancer:
- **Kubernetes**: LoadBalancer Service (built-in)
- **AWS**: Network Load Balancer (NLB)
- **GCP**: Network Load Balancer
- **Azure**: Load Balancer

## Troubleshooting

### Server won't start

1. **Check certificates exist**:
   ```bash
   ls -la certs/
   ```

2. **Check port availability**:
   ```bash
   lsof -i :5000
   ```

3. **Check logs**:
   ```bash
   docker-compose logs relay-server
   ```

### Clients can't connect

1. **Verify server is listening**:
   ```bash
   nc -zvu <server-ip> 5000
   ```

2. **Check firewall rules**:
   - Allow UDP port 5000 inbound
   - Security groups (AWS/GCP)
   - Network policies (Kubernetes)

3. **Test with local client**:
   ```bash
   cargo run -p multiplayer-pong -- --server <server-ip>:5000
   ```

### High latency

1. **Check server location**: Deploy server close to players
2. **Monitor resources**: CPU/memory usage
3. **Increase max_clients** if approaching limit

## Security

### Best Practices

1. **Use production TLS certificates** (Let's Encrypt)
2. **Enable firewall** rules (allow only port 5000 UDP)
3. **Set resource limits** in Kubernetes/Docker
4. **Monitor logs** for suspicious activity
5. **Update regularly** to latest ISSUN version
6. **Use private networks** when possible
7. **Implement rate limiting** (future feature)

### Network Policies (Kubernetes)

Restrict traffic to relay server:

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: issun-relay-policy
spec:
  podSelector:
    matchLabels:
      app: issun-relay
  policyTypes:
  - Ingress
  ingress:
  - from:
    - podSelector: {}
    ports:
    - protocol: UDP
      port: 5000
```

## Cost Optimization

### Resource Requests

Minimum resources for light load (<100 concurrent players):
- CPU: 100m
- Memory: 128Mi

Recommended for production (up to 1000 players):
- CPU: 500m
- Memory: 512Mi

### Spot Instances / Preemptible VMs

The server is stateless, so it's safe to use:
- AWS Spot Instances
- GCP Preemptible VMs
- Azure Spot VMs

### Auto-scaling

Configure HPA (Horizontal Pod Autoscaler) based on CPU:

```bash
kubectl autoscale deployment issun-relay-server \
  --cpu-percent=70 \
  --min=1 \
  --max=5
```

## Support

For deployment issues:
- GitHub Issues: https://github.com/ynishi/issun/issues
- Documentation: https://docs.rs/issun
