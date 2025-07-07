# ICN Production Security Guide

> **Securing your ICN Federation for Production Deployment**

This guide provides comprehensive security recommendations for deploying ICN nodes in production environments. It covers the Ed25519 migration, authentication setup, TLS configuration, and operational security best practices.

---

## 🛡️ **Security Overview**

ICN implements multiple layers of security:
- **Cryptographic Identity**: Ed25519-based DID authentication with memory protection
- **Transport Security**: TLS 1.3 encryption for all HTTP API communications
- **API Authentication**: Multi-layer authentication with API keys and bearer tokens
- **Audit Logging**: Comprehensive audit trails for all security-relevant operations
- **Resource Protection**: Mana-based rate limiting and economic enforcement

---

## 🔐 **Ed25519 Cryptographic Security**

### **Migration from StubSigner**

ICN has migrated from development `StubSigner` to production-grade `Ed25519Signer` with memory protection:

```rust
// Production deployment (automatic)
let runtime_context = RuntimeContext::new_with_real_libp2p(config).await?;

// Development/testing (manual)
let runtime_context = RuntimeContext::new_with_stub_signer(config).await?;
```

### **Ed25519Signer Features**

- **Real Cryptographic Signing**: Uses industry-standard Ed25519 signatures
- **Memory Protection**: Private keys are zeroized on drop to prevent memory leaks
- **Secure Key Generation**: Uses cryptographically secure random number generation
- **Performance Optimized**: Fast signing and verification operations

### **Key Management Best Practices**

1. **Generate Secure Keys**:
   ```bash
   # Generate a new Ed25519 keypair
   openssl genpkey -algorithm Ed25519 -out private_key.pem
   openssl pkey -in private_key.pem -pubout -out public_key.pem
   ```

2. **Store Keys Securely**:
   - Use hardware security modules (HSMs) for high-value keys
   - Implement key rotation schedules
   - Never store keys in version control
   - Use encrypted storage for key backups

3. **Key Rotation**:
   ```bash
   # Regular key rotation (recommended: every 90 days)
   icn-cli identity rotate-key --current-key-path ./current_key.pem
   ```

---

## 🔒 **API Authentication**

### **Multi-Layer Authentication**

ICN supports multiple authentication mechanisms that can be used together:

1. **API Key Authentication** (`x-api-key` header)
2. **Bearer Token Authentication** (`Authorization: Bearer <token>` header)
3. **Rate Limiting** for unauthenticated requests

### **Configuration Options**

```toml
# Production configuration
api_key = "your-secure-api-key-here"              # Required for all requests
auth_token = "your-bearer-token-here"             # Additional authentication layer
auth_token_path = "/secrets/bearer-token"         # Load token from file
open_rate_limit = 0                               # Disable unauthenticated requests
```

### **Secure Token Generation**

```bash
# Generate secure API key
openssl rand -hex 32

# Generate secure bearer token
openssl rand -base64 64
```

### **Authentication Best Practices**

1. **Use Both API Key and Bearer Token** in production
2. **Rotate Tokens Regularly** (recommended: every 30 days)
3. **Store Tokens Securely** (environment variables or secret management)
4. **Monitor Failed Authentication Attempts**
5. **Implement Token Expiration** and refresh mechanisms

### **Example Production Authentication**

```bash
# Start node with dual authentication
./icn-node \
  --api-key "$(cat /secrets/api-key)" \
  --auth-token-path "/secrets/bearer-token" \
  --open-rate-limit 0

# Client request with authentication
curl -X GET https://your-node.domain.com/status \
  -H "x-api-key: your-api-key" \
  -H "Authorization: Bearer your-bearer-token"
```

---

## 🔐 **TLS/HTTPS Configuration**

### **Built-in TLS Support**

ICN provides native TLS support using Rustls:

```bash
# Enable HTTPS with certificate files
./icn-node \
  --http-listen-addr 0.0.0.0:8443 \
  --tls-cert-path /etc/ssl/certs/server.crt \
  --tls-key-path /etc/ssl/private/server.key \
  --api-key "your-api-key"
```

### **Certificate Management**

#### **Option 1: Let's Encrypt (Recommended)**

```bash
# Install certbot
sudo apt-get install certbot

# Generate certificate
sudo certbot certonly --standalone -d your-node.domain.com

# Certificate files will be in:
# /etc/letsencrypt/live/your-node.domain.com/fullchain.pem
# /etc/letsencrypt/live/your-node.domain.com/privkey.pem
```

#### **Option 2: Self-Signed Certificates (Development)**

```bash
# Generate self-signed certificate
openssl req -x509 -newkey rsa:4096 -keyout server.key -out server.crt \
  -days 365 -nodes -subj "/CN=your-node.domain.com"
```

#### **Option 3: Corporate CA**

```bash
# Use your organization's CA to sign certificates
# Follow your organization's certificate request process
```

### **TLS Configuration Examples**

#### **Production with Let's Encrypt**

```toml
# production.toml
node_name = "Production Federation Node"
http_listen_addr = "0.0.0.0:8443"
tls_cert_path = "/etc/letsencrypt/live/your-node.domain.com/fullchain.pem"
tls_key_path = "/etc/letsencrypt/live/your-node.domain.com/privkey.pem"
api_key = "production-api-key"
auth_token = "production-bearer-token"
```

#### **Certificate Renewal Automation**

```bash
# Add to crontab for automatic renewal
0 12 * * * /usr/bin/certbot renew --quiet --post-hook "systemctl restart icn-node"
```

### **TLS Security Settings**

ICN enforces strong TLS settings by default:
- **TLS 1.3 Only**: No support for older, vulnerable TLS versions
- **Strong Cipher Suites**: Only AEAD ciphers supported
- **Perfect Forward Secrecy**: All key exchanges provide PFS
- **Certificate Validation**: Full certificate chain validation

---

## 🏗️ **Production Deployment Architecture**

### **Recommended Production Setup**

```
┌─────────────────────────────────────────────────────────────┐
│                    Load Balancer                            │
│                  (Nginx/HAProxy)                            │
│                   TLS Termination                           │
├─────────────────────────────────────────────────────────────┤
│                 ICN Node Cluster                            │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐   │
│  │   ICN Node A  │  │   ICN Node B  │  │   ICN Node C  │   │
│  │   Port 8080   │  │   Port 8080   │  │   Port 8080   │   │
│  └───────────────┘  └───────────────┘  └───────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                   Persistent Storage                        │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐   │
│  │   SQLite DB   │  │   Sled Store  │  │   RocksDB     │   │
│  └───────────────┘  └───────────────┘  └───────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### **Load Balancer Configuration (Nginx)**

```nginx
upstream icn_backend {
    server 127.0.0.1:8080;
    server 127.0.0.1:8081;
    server 127.0.0.1:8082;
}

server {
    listen 443 ssl http2;
    server_name your-node.domain.com;
    
    # TLS Configuration
    ssl_certificate /etc/letsencrypt/live/your-node.domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/your-node.domain.com/privkey.pem;
    ssl_protocols TLSv1.3;
    ssl_ciphers ECDHE+AESGCM:ECDHE+CHACHA20:DHE+AESGCM:DHE+CHACHA20:!aNULL:!MD5:!DSS;
    ssl_prefer_server_ciphers off;
    
    # Security Headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Content-Type-Options nosniff;
    add_header X-Frame-Options DENY;
    add_header X-XSS-Protection "1; mode=block";
    
    # Pass authentication headers
    location / {
        proxy_pass http://icn_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Pass authentication (if terminating at load balancer)
        # proxy_set_header x-api-key "your-api-key";
        # proxy_set_header Authorization "Bearer your-token";
    }
}
```

### **Systemd Service Configuration**

```ini
# /etc/systemd/system/icn-node.service
[Unit]
Description=ICN Federation Node
After=network.target
Wants=network.target

[Service]
Type=exec
User=icn
Group=icn
WorkingDirectory=/opt/icn
ExecStart=/opt/icn/bin/icn-node --config /etc/icn/production.toml
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=icn-node

# Security settings
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/icn
PrivateTmp=true
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true

# Environment
Environment=RUST_LOG=info,icn_node=debug,audit=info
EnvironmentFile=-/etc/icn/environment

[Install]
WantedBy=multi-user.target
```

### **File System Security**

```bash
# Create dedicated user and group
sudo useradd --system --home /var/lib/icn --shell /bin/false icn

# Set up directories with proper permissions
sudo mkdir -p /var/lib/icn/{data,logs}
sudo mkdir -p /etc/icn
sudo chown -R icn:icn /var/lib/icn
sudo chmod 700 /var/lib/icn
sudo chmod 600 /etc/icn/production.toml
```

---

## 📊 **Monitoring and Observability**

### **Security Metrics**

ICN exposes security-relevant metrics via Prometheus:

```bash
# Authentication failures
icn_auth_failures_total{reason="invalid_api_key"}
icn_auth_failures_total{reason="missing_bearer_token"}

# Rate limiting
icn_rate_limit_exceeded_total
icn_requests_rate_limited_total

# TLS connections
icn_tls_connections_total
icn_tls_handshake_errors_total
```

### **Audit Logging**

All security events are logged to the `audit` target:

```rust
// Enable audit logging
RUST_LOG=audit=info,icn_node=info

// Key audit events:
// - Authentication failures
// - Job submissions
// - Governance actions
// - Federation changes
// - Configuration changes
```

### **Log Monitoring Setup**

```bash
# Example with journald and fail2ban
sudo journalctl -u icn-node -f | grep "auth_failed"

# Fail2ban configuration for ICN
[icn-auth]
enabled = true
port = 8443
protocol = tcp
filter = icn-auth
logpath = /var/log/journal
maxretry = 5
bantime = 3600
```

### **Security Dashboards**

Grafana dashboard for ICN security monitoring:

```json
{
  "dashboard": {
    "title": "ICN Security Monitoring",
    "panels": [
      {
        "title": "Authentication Failures",
        "type": "stat",
        "targets": [
          {
            "expr": "rate(icn_auth_failures_total[5m])"
          }
        ]
      },
      {
        "title": "TLS Connection Health",
        "type": "graph",
        "targets": [
          {
            "expr": "icn_tls_connections_total"
          }
        ]
      }
    ]
  }
}
```

---

## 🛡️ **Security Hardening Checklist**

### **Network Security**

- [ ] **Firewall Configuration**: Only allow necessary ports (443, P2P port)
- [ ] **DDoS Protection**: Implement rate limiting and connection limits
- [ ] **Network Segmentation**: Isolate ICN nodes in secure network zones
- [ ] **VPN Access**: Require VPN for administrative access

### **Host Security**

- [ ] **OS Hardening**: Apply CIS benchmarks for your OS
- [ ] **Regular Updates**: Keep OS and dependencies updated
- [ ] **Minimal Services**: Disable unnecessary services
- [ ] **File Integrity**: Monitor critical files with AIDE/Tripwire

### **Application Security**

- [ ] **Authentication**: Enable both API key and bearer token auth
- [ ] **TLS Configuration**: Use TLS 1.3 with strong ciphers
- [ ] **Rate Limiting**: Configure appropriate rate limits
- [ ] **Audit Logging**: Enable comprehensive audit logging
- [ ] **Regular Updates**: Keep ICN software updated

### **Operational Security**

- [ ] **Key Rotation**: Implement regular key rotation schedule
- [ ] **Backup Security**: Encrypt and secure backup data
- [ ] **Incident Response**: Develop incident response procedures
- [ ] **Security Monitoring**: Set up alerting for security events

### **Data Protection**

- [ ] **Data Encryption**: Encrypt data at rest and in transit
- [ ] **Access Controls**: Implement principle of least privilege
- [ ] **Data Retention**: Define and implement data retention policies
- [ ] **Privacy Compliance**: Ensure compliance with privacy regulations

---

## 🚨 **Incident Response**

### **Security Event Response**

1. **Detection**: Monitor audit logs and metrics for anomalies
2. **Assessment**: Determine severity and scope of incident
3. **Containment**: Isolate affected systems if necessary
4. **Investigation**: Analyze logs and gather evidence
5. **Recovery**: Restore normal operations securely
6. **Lessons Learned**: Update procedures based on incident

### **Common Security Incidents**

#### **Authentication Attacks**

```bash
# Monitor for authentication failures
journalctl -u icn-node | grep "auth_failed" | tail -100

# Temporary mitigation
sudo ufw deny from <attacker-ip>
```

#### **TLS/Certificate Issues**

```bash
# Check certificate expiration
openssl x509 -in /etc/ssl/certs/server.crt -text -noout | grep "Not After"

# Verify certificate chain
openssl verify -CAfile /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/server.crt
```

#### **Resource Exhaustion**

```bash
# Check system resources
top -p $(pgrep icn-node)
netstat -an | grep :8443 | wc -l

# Adjust rate limits if necessary
icn-cli config set open_rate_limit 10
```

### **Emergency Contacts**

- **Security Team**: security@intercooperative.network
- **On-Call Engineer**: [Your 24/7 contact]
- **ICN Community**: [GitHub Discussions](https://github.com/InterCooperative-Network/icn-core/discussions)

---

## 📚 **Security Resources**

### **Documentation**
- [ICN API Security](API.md#security-considerations)
- [Governance Security Framework](governance-framework.md#security--integrity)
- [Federation Security Guide](federation-security.md)

### **Security Tools**
- [Rustls TLS Library](https://github.com/rustls/rustls)
- [Ring Cryptography](https://github.com/briansmith/ring)
- [Zeroize Memory Protection](https://github.com/RustCrypto/utils/tree/master/zeroize)

### **Compliance Resources**
- [NIST Cybersecurity Framework](https://www.nist.gov/cyberframework)
- [CIS Controls](https://www.cisecurity.org/controls/)
- [OWASP Application Security](https://owasp.org/)

### **Community Security**
- **Security Advisories**: [GitHub Security](https://github.com/InterCooperative-Network/icn-core/security)
- **Bug Bounty Program**: [security@intercooperative.network](mailto:security@intercooperative.network)
- **Security Research**: [Research Partnerships](research-partnerships.md)

---

## ⚡ **Quick Start: Secure Production Deployment**

```bash
# 1. Generate secure credentials
API_KEY=$(openssl rand -hex 32)
BEARER_TOKEN=$(openssl rand -base64 64)

# 2. Obtain TLS certificate (Let's Encrypt)
sudo certbot certonly --standalone -d your-node.domain.com

# 3. Create secure configuration
cat > /etc/icn/production.toml << EOF
node_name = "Production Node"
http_listen_addr = "0.0.0.0:8443"
storage_backend = "sqlite"
storage_path = "/var/lib/icn/data/node.sqlite"
mana_ledger_backend = "sled"
mana_ledger_path = "/var/lib/icn/data/mana.sled"
tls_cert_path = "/etc/letsencrypt/live/your-node.domain.com/fullchain.pem"
tls_key_path = "/etc/letsencrypt/live/your-node.domain.com/privkey.pem"
api_key = "$API_KEY"
auth_token = "$BEARER_TOKEN"
open_rate_limit = 0
enable_p2p = true
p2p_listen_addr = "/ip4/0.0.0.0/tcp/4001"
EOF

# 4. Start secure node
sudo systemctl start icn-node
sudo systemctl enable icn-node

# 5. Verify security
curl -H "x-api-key: $API_KEY" -H "Authorization: Bearer $BEARER_TOKEN" \
  https://your-node.domain.com/status
```

---

**🔒 Remember: Security is an ongoing process, not a one-time setup. Regularly review and update your security configuration as the ICN ecosystem evolves.** 