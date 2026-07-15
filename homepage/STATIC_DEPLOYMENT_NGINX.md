# Static Nginx Deployment (No Docker)

Deploy the pre-built static site to your Ubuntu server with Nginx.

## Option 1: Build Locally, Upload `dist/` Only (Recommended)

### On Your Dev Machine

```bash
cd ~/hirn/homepage

# Set your domain for build
export SITE_URL="https://your-domain.com"

# Install dependencies (one-time)
bun install

# Build static site
bun run build

# This creates dist/ folder with all HTML/CSS/JS ready to serve
ls -la dist/  # ~10-50MB depending on content
```

### Upload to Server

```bash
# Upload the contents of the dist/ folder (fast, ~10-50MB)
# Note: trailing slash on dist/ with rsync copies the contents of dist/ into the destination folder
rsync -avz --progress dist/ user@your-server-ip:/etc/www/hirn-home/

# Or with scp (note the trailing dot to copy directory contents instead of the directory itself)
scp -r dist/. user@your-server-ip:/etc/www/hirn-home/
```

### On Ubuntu Server: Configure Nginx

```bash
ssh user@your-server-ip

# Create site directory
sudo mkdir -p /etc/www/hirn-home

# Set permissions (if needed)
sudo chown user:user /etc/www/hirn-home

# Create Nginx config
sudo tee /etc/nginx/sites-available/hirn > /dev/null <<'EOF'
server {
    listen 80;
    server_name your-domain.com www.your-domain.com;

    root /etc/www/hirn-home;
    index index.html;

    # SPA routing: serve index.html for missing files
    location / {
        try_files $uri $uri/ /index.html;
    }

    # Cache static assets for 30 days
    location ~* \.(css|js|mjs|map|json|svg|ico|png|jpg|jpeg|gif|webp|avif|woff2?)$ {
        expires 30d;
        add_header Cache-Control "public, max-age=2592000, immutable";
    }

    # Compress text responses
    gzip on;
    gzip_types text/plain text/css application/json application/javascript text/xml application/xml+rss text/javascript;
}
EOF

# Enable the site
sudo ln -s /etc/nginx/sites-available/hirn /etc/nginx/sites-enabled/

# Test config
sudo nginx -t

# Restart Nginx
sudo systemctl restart nginx

# Verify it's running
sudo systemctl status nginx
curl http://your-domain.com
```

---

## Option 2: Build on Server (Requires Bun)

If you want to build on the server itself:

### On Ubuntu Server: Install Bun

```bash
# Install Bun (fast runtime & package manager)
curl -fsSL https://bun.sh/install | bash

# Verify
bun --version
```

### Upload Entire Repository

```bash
# From your dev machine, upload the homepage folder contents (excluding build/temp files)
rsync -avz \
  --exclude='node_modules' \
  --exclude='dist' \
  --exclude='.astro' \
  --exclude='.git' \
  --exclude='*.log' \
  ~/hirn/homepage/ user@your-server-ip:/etc/www/hirn-home/
```

### Build on Server

```bash
ssh user@your-server-ip

cd /etc/www/hirn-home

# Set domain
export SITE_URL="https://your-domain.com"

# Install dependencies
bun install

# Build
bun run build

# dist/ folder is now ready to serve
```

### Configure Nginx (same as Option 1)

See "Configure Nginx" section above.

---

## Option 3: Git-Based Deployment (Auto-Update)

Best for automated content updates.

### On Ubuntu Server

```bash
# Clone repository sparsely (HEAD only) to only get the homepage folder
cd /etc/www
git clone --depth 1 --sparse --filter=blob:none https://github.com/hirnlabs/hirn.git hirn-home
cd hirn-home
git sparse-checkout set homepage
cd homepage

# Install dependencies
bun install --frozen-lockfile

# Initial build
export SITE_URL="https://your-domain.com"
bun run build
```

### Create Update Script

```bash
# Create ~/update-blog.sh
cat > ~/update-blog.sh <<'SCRIPT'
#!/bin/bash
set -e

cd /etc/www/hirn-home/homepage

# Pull latest changes
git pull origin main

# Install (in case deps changed)
bun install

# Rebuild
export SITE_URL="https://your-domain.com"
bun run build

echo "✅ Blog updated and rebuilt"
SCRIPT

chmod +x ~/update-blog.sh
```

### Automated Updates with Cron

```bash
# Edit crontab
crontab -e

# Add line to rebuild every 6 hours (adjust as needed)
0 */6 * * * ~/update-blog.sh >> ~/blog-update.log 2>&1

# Or rebuild when you push to GitHub via webhook
# (requires setting up a webhook receiver)
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
   Push your changes to the `hirn_desktop` branch. GitHub Actions will trigger, build, and deploy the updated static site directly into `/etc/www/hirn-home/`.

---

## Nginx Configuration: Full Example

**File: `/etc/nginx/sites-available/hirn`**

```nginx
# HTTP to HTTPS redirect (if using SSL)
server {
    listen 80;
    server_name your-domain.com www.your-domain.com;
    return 301 https://$server_name$request_uri;
}

# HTTPS server
server {
    listen 443 ssl http2;
    server_name your-domain.com www.your-domain.com;

    # SSL certificates (use certbot for free Let's Encrypt)
    ssl_certificate /etc/letsencrypt/live/your-domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/your-domain.com/privkey.pem;

    # Performance
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;

    root /etc/www/hirn-home;
    index index.html;

    # SPA routing
    location / {
        try_files $uri $uri/ /index.html;
    }

    # Cache static assets
    location ~* \.(css|js|mjs|map|json|svg|ico|png|jpg|jpeg|gif|webp|avif|woff2?)$ {
        expires 30d;
        add_header Cache-Control "public, max-age=2592000, immutable";
    }

    # Compression
    gzip on;
    gzip_types text/plain text/css application/json application/javascript;
    gzip_min_length 1000;

    # Deny access to dotfiles
    location ~ /\. {
        deny all;
    }
}
```

---

## Enable HTTPS with Let's Encrypt (Free)

```bash
# Install certbot
sudo apt install -y certbot python3-certbot-nginx

# Get certificate (auto-configures Nginx)
sudo certbot --nginx -d your-domain.com -d www.your-domain.com

# Auto-renewal (certbot handles this, but verify)
sudo systemctl enable certbot.timer
sudo systemctl start certbot.timer

# Test renewal
sudo certbot renew --dry-run
```

---

## Quick Deployment Checklist

- [ ] Build locally: `SITE_URL=https://your-domain.com bun run build`
- [ ] Upload `dist/` folder to server
- [ ] Create Nginx config in `/etc/nginx/sites-available/`
- [ ] Enable site: `sudo ln -s /etc/nginx/sites-available/hirn /etc/nginx/sites-enabled/`
- [ ] Test: `sudo nginx -t`
- [ ] Restart: `sudo systemctl restart nginx`
- [ ] Set up SSL with `certbot`
- [ ] Test site: `curl https://your-domain.com`

---

## Troubleshooting

### 404 errors on page reload
**Problem:** Navigating directly to URLs like `/note/something` shows 404  
**Fix:** Ensure `try_files $uri $uri/ /index.html;` is in your Nginx config

### CSS/JS not loading
**Problem:** Styles/scripts broken after deploy  
**Fix:** Clear browser cache (Ctrl+Shift+Del) or use hard refresh (Ctrl+F5)

### Nginx won't start
```bash
# Check for syntax errors
sudo nginx -t

# View logs
sudo tail -f /var/log/nginx/error.log
```

### Update dist/ without restarting Nginx
```bash
# Just replace the dist/ folder
rsync -avz --delete dist/ user@server:/etc/www/hirn-home/

# Nginx will serve new files immediately (no restart needed)
```

---

## File Structure on Server

```
/etc/www/hirn-home/
├── index.html              # Homepage
├── en/                     # English pages
├── ja/                     # Japanese pages
├── zh-cn/                  # Chinese pages
├── _astro/                 # JS/CSS bundles
├── fonts/                  # Web fonts
├── feed.xml                # RSS feed
├── sitemap-index.xml       # Sitemap
└── ...other static files
```

**No Node.js needed to serve** — just static files. Nginx is a simple HTTP server.

---

## Comparison: Option 1 vs Option 2 vs Option 3

| | Option 1 | Option 2 | Option 3 |
|---|---|---|---|
| **Build location** | Dev machine | Server | Server (auto) |
| **Server load** | Low | Medium | Medium |
| **Deployment speed** | Fast (10-50MB) | Slow (300+ MB deps) | Slow (first time) |
| **Best for** | Stable content | Frequent changes | Git-based workflow |
| **Requires Node.js** | No | Yes | Yes |
| **Auto-update** | Manual | Manual | Cron/webhook |

**Recommendation:** Use **Option 1** for simplicity, **Option 3** if you push content often via Git.
