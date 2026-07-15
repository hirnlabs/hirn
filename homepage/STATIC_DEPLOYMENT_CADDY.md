# Static Caddy Deployment (No Docker)

Deploy the pre-built static site to your Ubuntu server with Caddy. Caddy automatically handles SSL/HTTPS certificates and renewal for you.

---

## Option 1: Build Locally, Upload `dist/` Only (Recommended)

This method keeps your production server lightweight since you don't need to install Node.js/Bun or run builds on it.

### 1. On Your Dev Machine

Navigate to the `homepage` directory, set the domain, install dependencies, and build:

```bash
cd ~/hirn/homepage

# Set your domain for the build
export SITE_URL="https://your-domain.com"

# Install dependencies (one-time)
bun install

```

### 2. Upload to Server

Copy the **contents** of the `dist/` directory to the server:

```bash
# Using rsync (copies the contents of dist/ into /etc/www/hirn-home/)
rsync -avz --progress dist/ user@your-server-ip:/etc/www/hirn-home/

# Or using scp (note the dot '.' to copy contents rather than the directory itself)
scp -r dist/. user@your-server-ip:/etc/www/hirn-home/
```

### 3. On Ubuntu Server: Configure Caddy

SSH into your server and set up the Caddyfile:

```bash
ssh user@your-server-ip

# Create site directory (if not exists)
sudo mkdir -p /etc/www/hirn-home
sudo chown user:user /etc/www/hirn-home

# Create Caddyfile
sudo tee /etc/caddy/Caddyfile > /dev/null <<'EOF'
your-domain.com {
    # Root folder containing your index.html
    root * /etc/www/hirn-home

    # Enable Gzip compression
    encode gzip

    # Serve static files
    file_server

    # SPA routing: fallback to check file, directory, or return 404
    try_files {path} {path}/ =404

    # Cache static assets for 30 days
    @static path_regexp static \.(?:css|js|mjs|map|json|svg|ico|png|jpg|jpeg|gif|webp|avif|woff2?)$
    header @static Cache-Control "public, max-age=2592000, immutable"
}
EOF

# Format the Caddyfile
sudo caddy fmt --overwrite /etc/caddy/Caddyfile

# Reload Caddy config (no downtime)
sudo systemctl reload caddy
```

---

## Option 2: Build on Server (Requires Bun)

If you prefer building directly on the server:

### 1. Install Caddy & Bun on Server

```bash
# Install Caddy on Ubuntu
sudo apt install -y debian-keyring debian-archive-keyring apt-transport-https curl
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy-stable.list
sudo apt update
sudo apt install caddy -y

# Install Bun (fast runtime & package manager)
curl -fsSL https://bun.sh/install | bash
```

### 2. Upload Project Code

Upload only the `homepage/` directory files (excluding node_modules/dist):

```bash
rsync -avz \
  --exclude='node_modules' \
  --exclude='dist' \
  --exclude='.astro' \
  --exclude='.git' \
  --exclude='*.log' \
  ~/hirn/homepage/ user@your-server-ip:/etc/www/hirn-home/
```

### 3. Build on Server

```bash
ssh user@your-server-ip
cd /etc/www/hirn-home

# Build the site
export SITE_URL="https://your-domain.com"
bun install
bun run build
```

### 4. Configure Caddyfile

Create the Caddyfile targeting the built `dist` folder:

```bash
sudo tee /etc/caddy/Caddyfile > /dev/null <<'EOF'
your-domain.com {
    # Root points to the dist directory
    root * /etc/www/hirn-home/dist
    
    encode gzip
    file_server
    try_files {path} {path}/ =404

    @static path_regexp static \.(?:css|js|mjs|map|json|svg|ico|png|jpg|jpeg|gif|webp|avif|woff2?)$
    header @static Cache-Control "public, max-age=2592000, immutable"
}
EOF

sudo systemctl reload caddy
```

---

## Option 3: Git-Based Deployment (Auto-Update)

Best for automated updates from GitHub.

### 1. On Ubuntu Server

Clone only the `homepage/` folder using Git sparse-checkout:

```bash
cd /etc/www
git clone --depth 1 --sparse --filter=blob:none https://github.com/hirnlabs/hirn.git hirn-home
cd hirn-home
git sparse-checkout set homepage
cd homepage

# Install deps and build
bun install
export SITE_URL="https://your-domain.com"
bun run build
```

### 2. Configure Caddyfile

```bash
sudo tee /etc/caddy/Caddyfile > /dev/null <<'EOF'
your-domain.com {
    # Root points to the built dist folder inside homepage
    root * /etc/www/hirn-home/homepage/dist
    
    encode gzip
    file_server
    try_files {path} {path}/ =404

    @static path_regexp static \.(?:css|js|mjs|map|json|svg|ico|png|jpg|jpeg|gif|webp|avif|woff2?)$
    header @static Cache-Control "public, max-age=2592000, immutable"
}
EOF

sudo systemctl reload caddy
```

### 3. Create Update Script

Create a script `~/update-blog.sh` to fetch and rebuild changes:

```bash
cat > ~/update-blog.sh <<'SCRIPT'
#!/bin/bash
set -e

cd /etc/www/hirn-home/homepage

# Pull latest changes from HEAD
git pull origin main

# Re-install dependencies if package.json updated
bun install

# Rebuild static site
export SITE_URL="https://your-domain.com"
bun run build

echo "✅ Website successfully updated and rebuilt"
SCRIPT

chmod +x ~/update-blog.sh
```

Add to cron (`crontab -e`) to run periodically (e.g., every 6 hours):
```text
0 */6 * * * ~/update-blog.sh >> ~/blog-update.log 2>&1
```

---

## Option 4: GitHub Actions Auto-Deployment (CI/CD)

For fully automated deployments on every git push, you can use the pre-configured GitHub Actions workflow located in `.github/workflows/deploy.yaml`.

This workflow will:
1. Trigger automatically when you push to the `hirn_desktop` branch.
2. Checkout your code and set up Bun.
3. Install dependencies (`bun install --frozen-lockfile`) and build the site (`bun run build`).
4. Securely upload the static `dist/` directory contents to your server using SSH/SCP.

### Setup Steps

1. **Configure Repository Secrets**:
   On GitHub, go to your repository **Settings > Secrets and variables > Actions** and add the following repository secrets:
   * `SSH_HOST`: The IP address or hostname of your production server.
   * `SSH_USER`: The SSH username (e.g., `ubuntu` or `root`).
   * `SSH_PRIVATE_KEY`: The private SSH key used to access your server (must correspond to an authorized key on the server).

2. **Server Directory Preparation**:
   Ensure the destination directory exists and has the correct permissions for the SSH user:
   ```bash
   sudo mkdir -p /etc/www/hirn-home
   sudo chown -R your-ssh-user:your-ssh-user /etc/www/hirn-home
   ```

3. **Deploy**:
   Push your changes to the `hirn_desktop` branch. GitHub Actions will trigger, build, and deploy the updated static site directly into `/etc/www/hirn-home/`. Caddy serves the updated files instantly.

---

## Quick Troubleshooting

### Changes not reflecting
Caddy serves files instantly after rebuilding. If changes don't show up, try a hard refresh in the browser (`Ctrl + F5` or `Cmd + Shift + R`) to bypass cached static assets.

### Verify Caddy status
```bash
# Check Caddy logs
sudo journalctl -u caddy --no-pager -n 50

# Test Caddyfile syntax
sudo caddy validate --config /etc/caddy/Caddyfile
```
