# Rwatch Kubernetes Deployment Guide

This guide explains how to deploy rwatch to your Kubernetes cluster using GitHub Actions with GitHub App authentication (following the jakob-lingel pattern).

## Prerequisites

1. **GitHub App**: You need a GitHub App with:
   - Read/write access to both `rwatch` and `app-of-apps` repositories
   - `APP_ID` and `APP_PRIVATE_KEY` secrets configured in the rwatch repo

2. **Cluster Access**: Your K3s cluster must be running with:
   - ArgoCD installed and configured
   - NGINX Ingress Controller
   - cert-manager for TLS

## Files Created

### 1. GitHub Actions Workflows (in rwatch repo)

**`.github/workflows/main.yml`**
- Triggers on push to main or manual dispatch
- Bumps version in Cargo.toml files
- Builds and pushes Docker image to GHCR
- Updates app-of-apps with new image tag
- Uses GitHub App token instead of PAT

**`.github/workflows/build-and-push.yml`**
- Reusable workflow for building and pushing Docker images
- Reads version from Cargo.toml
- Tags images with: `latest`, `sha-<commit>`, `<version>`

### 2. Deployment Manifests (in app-of-apps repo)

**`rwatch/` directory:**
- `rwatch-namespace.yaml` - rwatch namespace
- `rwatch-daemonset.yaml` - Agent DaemonSet (runs on all nodes)
- `rwatch-service.yaml` - Headless service for agent discovery

**`rwatch-app.yaml`** (in app-of-apps root):
- ArgoCD Application resource
- Auto-sync enabled with prune and self-heal

## Setup Instructions

### Step 1: Configure GitHub Secrets

In the rwatch repository, add these secrets:

1. `APP_ID` - Your GitHub App ID
2. `APP_PRIVATE_KEY` - Your GitHub App's private key (PEM format)

### Step 2: Commit Workflow Files

In your rwatch repository:

```bash
# Add the workflow files
git add .github/workflows/
git add Dockerfile.agent

# Commit
git commit -m "Add GitHub Actions workflows with GitHub App"
git push origin main
```

### Step 3: Create app-of-apps Directory

In your app-of-apps repository:

```bash
mkdir -p rwatch
cd rwatch

# Create the deployment manifests (already done)
# Then commit:
git add rwatch/
git add rwatch-app.yaml
git commit -m "Add rwatch monitoring deployment"
git push origin main
```

### Step 4: ArgoCD Will Auto-Deploy

ArgoCD will automatically:
1. Detect the new rwatch-app.yaml
2. Create the rwatch namespace
3. Deploy the DaemonSet (one agent pod per node)
4. Create the headless service

### Step 5: Verify Deployment

```bash
# Check if agents are running on all nodes
kubectl get pods -n rwatch -o wide

# Check agent health
kubectl exec -n rwatch <agent-pod-name> -- wget -qO- http://localhost:3000/health

# View agent logs
kubectl logs -n rwatch -l app=rwatch-agent
```

## Accessing Rwatch Data

### Option 1: Run TUI Locally (Recommended for Development)

```bash
# Port-forward to access agents from your local machine
kubectl port-forward -n rwatch service/rwatch-agent 3000:3000

# In another terminal, run the TUI
cd /path/to/rwatch
cargo run -p rwatch-tui
```

### Option 2: Access via Service DNS

From within the cluster, agents are accessible at:
```
rwatch-agent.rwatch.svc.cluster.local:3000
```

Each pod also has individual DNS:
```
<pod-ip>.rwatch-agent.rwatch.svc.cluster.local
```

### Option 3: Deploy TUI in Cluster (Future)

Create a TUI deployment that runs in the cluster and queries agents via the headless service.

## Deployment Flow

```
Push to rwatch/main
        │
        ▼
┌─────────────────────────────┐
│ GitHub Actions Workflow     │
│ - Check for changes         │
│ - Bump version              │
│ - Build Docker image        │
│ - Push to GHCR              │
│ - Update app-of-apps        │
└──────────────┬──────────────┘
               │
               │ GitHub App Token
               ▼
┌─────────────────────────────┐
│ app-of-apps repo            │
│ - DaemonSet updated         │
└──────────────┬──────────────┘
               │
               │ ArgoCD Sync
               ▼
┌─────────────────────────────┐
│ K3s Cluster                 │
│ - Agent running on nodes    │
│ - Memory metrics available  │
└─────────────────────────────┘
```

## Troubleshooting

### Agents not starting

```bash
# Check pod status
kubectl describe pods -n rwatch

# Check logs
kubectl logs -n rwatch -l app=rwatch-agent --previous

# Check node resources
kubectl top nodes
```

### Image pull errors

Ensure the image is public or configure image pull secrets:
```bash
# Check if image exists
docker pull ghcr.io/davidcode2/rwatch-agent:<version>
```

### ArgoCD sync issues

```bash
# Check ArgoCD application status
argocd app get rwatch

# Sync manually if needed
argocd app sync rwatch
```

## Security Notes

- Agents run with `hostPID: true` to access host /proc/meminfo
- Agents use `hostNetwork: true` for node-level networking
- No authentication on agent endpoints (internal cluster only)
- Agents run with read-only root filesystem
- All capabilities are dropped

## Architecture

```
┌─────────────────────────────────────────────┐
│              K3s Cluster                    │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐    │
│  │  Node 1  │ │  Node 2  │ │  Node 3  │    │
│  │ Agent:3000│ │ Agent:3000│ │ Agent:3000│   │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘    │
│       │            │            │           │
│       └────────────┼────────────┘           │
│                    │                        │
│         ┌────────▼─────────┐              │
│         │ Headless Service │              │
│         │ rwatch-agent    │              │
│         └─────────────────┘              │
└─────────────────────────────────────────────┘
```

## Next Steps

1. **Test the deployment** - Push a change to main and verify the workflow runs
2. **Monitor agents** - Check that agents are healthy on all nodes
3. **Access metrics** - Run TUI locally or deploy it in the cluster
4. **Add alerts** - Consider adding alerts for failed agents
5. **Web UI** - Future: Create a web interface using the client library
