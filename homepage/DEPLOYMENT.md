# Deployment Guide

## Quick Docker Deployment (Ubuntu)

### Static-Only Mode (Recommended for most users)

```bash
# 1. Clone repository sparsely (HEAD only) and enter homepage directory
git clone --depth 1 --sparse --filter=blob:none https://github.com/hirnlabs/hirn.git
cd hirn
git sparse-checkout set homepage
cd homepage

# 2. Configure your domain
echo "SITE_URL=https://your-domain.com" > .env

# 3. Build and run with Docker
docker compose up -d --build

# Container will serve on port 80
# Check logs: docker compose logs -f
```

### Behind Reverse Proxy (Nginx/Apache)

If you want the container on a different port (e.g., 8080) and reverse proxy from your main web server:

```bash
# Edit docker-compose.yml to change port mapping:
# - "8080:80"  # instead of "80:80"

docker compose up -d --build

# Then configure your main nginx/apache to proxy traffic:
# nginx example:
#   server_name your-domain.com;
#   location / {
#     proxy_pass http://localhost:8080;
#   }
```

## Ubuntu Server Setup

### Prerequisites

```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Install Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose

# Verify
docker --version
docker compose --version
```

### Deploy (non-root user)

```bash
# Add your user to docker group (so you don't need sudo)
sudo usermod -aG docker $USER
newgrp docker

# Clone repository sparsely (HEAD only) to only get the homepage folder
git clone --depth 1 --sparse --filter=blob:none https://github.com/hirnlabs/hirn.git ~/hirn
cd ~/hirn
git sparse-checkout set homepage
cd homepage

# Set environment
cp .env.example .env
nano .env  # Edit SITE_URL to your domain

# Deploy
docker compose up -d --build

# Check it's running
docker ps
curl http://localhost  # Should see HTML
```

## Updating After Git Pull

```bash
# Get latest changes
git pull origin main

# Rebuild and restart
docker compose up -d --build

# Clean up old images (optional)
docker image prune -f
```

## Troubleshooting

### Container won't start
```bash
# Check logs
docker compose logs focccus-home

# Common issue: SITE_URL not set
docker compose down
echo "SITE_URL=https://your-domain.com" > .env
docker compose up -d --build
```

### Port already in use
```bash
# Change port in docker-compose.yml to "8080:80"
# Then restart
docker compose down
docker compose up -d --build
```

### Clear build cache
```bash
docker compose down
docker image rm focccus-home:latest
docker compose up -d --build
```

## Monitoring

### View live logs
```bash
docker compose logs -f focccus-home
```

### Disk usage
```bash
docker system df
docker system prune -a  # Remove unused images/containers
```

## Backup & Recovery

Content is stored in `src/content/` on the host (not in the container):

```bash
# Backup content
tar -czf content-backup-$(date +%s).tar.gz src/content/

# The container can be destroyed and rebuilt without losing content
docker compose down
docker compose up -d --build
# All content preserved!
```

## SSL/HTTPS

For HTTPS, use a reverse proxy (nginx, Caddy, HAProxy) or services like Cloudflare. The container itself serves on HTTP only.

Example with Caddy (SSL auto-renewal):

Create a `Caddyfile` on your server:

```caddy
your-domain.com {
    reverse_proxy localhost:80
}
```

Run Caddy using Docker or install it as a system service on Ubuntu. Caddy will handle SSL certificates automatically.

## Production Checklist

- [ ] Set `SITE_URL` to your actual domain
- [ ] Configure firewall to allow inbound port 80 (or your proxy port)
- [ ] Set up SSL with reverse proxy or Cloudflare
- [ ] Configure automatic backups of `src/content/`
- [ ] Enable Docker auto-restart: `restart: unless-stopped` (already in compose file)
- [ ] Test `docker compose up` after reboot to verify auto-start works
- [ ] Monitor disk space (`df -h`, `docker system df`)
- [ ] Set up log rotation if needed
