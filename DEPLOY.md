# Deployment

## Hosting for friends

This is a **single-server, trusted-friends foundation**, not a public service: no room expiry, member removal/host transfer, game spectating, or abuse rate limiting. Benched members see room information, not a game view. Room codes are invitations, not secrets; member tokens are secrets. Skyjo plays one round, not a cumulative match to 100.

Run exactly one backend process against a database. Back up the database; it contains hidden game state and membership credentials. Stored rooms require the current data model; database migrations are not provided. Check release changes before upgrading and use a fresh database for incompatible saved data.

## Docker Compose with host nginx

The frontend container serves the built SPA. The backend container serves `/api`, including WebSockets. Host nginx routes both through one public origin and terminates TLS. The browser uses relative `/api` URLs, so no frontend environment variables or domain-specific rebuilds are needed.

Prerequisites: a Linux server with Git, Docker Engine **28+**, the Docker Compose v2 plugin, and nginx installed. Use a maintained Docker release; older releases can expose localhost-published ports to the local network. Point your domain's DNS A record (and AAAA record if using IPv6) at the server. Allow inbound TCP ports 80 and 443; do not expose container ports publicly.

### 1. Configure and start containers

Clone this repository onto the server, then run from its root:

```sh
cp .env.example .env
# Edit .env: replace cards.example.org with your domain.
docker compose up -d --build
docker compose ps
curl --fail http://127.0.0.1:3000/api/health
curl --fail --head http://127.0.0.1:8080/
```

The first build compiles Rust and the frontend; the host needs neither Rust nor Node. Both runtime containers run as non-root users. SQLite lives in the named `arena-data` volume mounted at `/data`; container replacement does not remove it. Compose prefixes the volume name with the project name: keep the repository directory name (or your chosen `docker compose -p NAME`) consistent across deployments.

Compose reads these settings from `.env` (shell environment variables take precedence):

| Variable | Compose default | Purpose |
| --- | --- | --- |
| `PUBLIC_ORIGIN` | Required | Exact browser origin, e.g. `https://cards.example.org`; scheme, hostname, and non-default port, with no path or trailing slash. Used to validate WebSocket origins. |
| `FRONTEND_PORT` | `8080` | Host loopback port for the frontend. |
| `BACKEND_PORT` | `3000` | Host loopback port for the backend. |
| `DATABASE_URL` | `sqlite:///data/arena.db` | SQLite file; keep it under `/data` for persistence. |
| `RUST_LOG` | `arena_server=info,tower_http=info` | Backend logging filter. Avoid verbose logging that could expose private data. |

Apply `.env` changes with `docker compose up -d`. Container-internal ports stay fixed; `BIND_ADDR` is deliberately set to `0.0.0.0:3000` so Docker can reach the backend. Changing a host port also requires changing nginx's upstream address below. `.env` is ignored by Git.

### 2. Route requests through nginx

On Debian/Ubuntu, save this as `/etc/nginx/sites-available/friends-arena`, replacing the domain. The `map` must be in nginx's `http` context; Debian/Ubuntu includes site files there. If your nginx already defines this map, reuse it instead of defining it twice.

```nginx
map $http_upgrade $connection_upgrade {
    default upgrade;
    ''      close;
}

server {
    listen 80;
    listen [::]:80;
    server_name cards.example.org;

    location = /api {
        return 308 /api/;
    }

    location /api/ {
        # No trailing slash: preserve /api in the upstream request.
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection $connection_upgrade;
        proxy_read_timeout 3600s;
        proxy_send_timeout 3600s;
        proxy_buffering off;
    }

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

Enable and validate it (remove or adjust a conflicting site for the same domain first):

```sh
sudo ln -s /etc/nginx/sites-available/friends-arena /etc/nginx/sites-enabled/friends-arena
sudo nginx -t
sudo systemctl reload nginx
```

On other distributions, put the configuration in a file included from nginx's `http` block, such as `/etc/nginx/conf.d/friends-arena.conf`. If nginx runs on another machine, loopback upstreams will not work: use a private network or tunnel and restrict access to nginx rather than exposing the API publicly.

### 3. Obtain an HTTPS certificate

Use a trusted TLS certificate before inviting players: HTTP exposes member tokens and game traffic. Let's Encrypt provides free certificates. On Debian/Ubuntu, one option is Certbot's nginx plugin:

```sh
sudo apt-get update
sudo apt-get install certbot python3-certbot-nginx
sudo certbot --nginx -d cards.example.org --redirect
sudo nginx -t
sudo systemctl reload nginx
sudo certbot renew --dry-run
```

Certbot adds the HTTPS listener and certificate paths and redirects HTTP to HTTPS. Verify its renewal timer is enabled (`systemctl list-timers --all`); certificates expire and renewal must reload nginx. HTTP-01 validation needs publicly reachable port 80 and correct DNS, including any AAAA record. For a private server, wildcard certificate, or blocked port 80, use DNS-01 validation with your DNS provider instead. Keep DNS API credentials private. If you already manage certificates, configure nginx's port 443 listener with your certificate/full chain and private key and keep the same proxy locations.

Set `PUBLIC_ORIGIN=https://cards.example.org` in `.env` and run `docker compose up -d` if not already set. TLS ends at nginx; loopback container traffic remains HTTP. HTTPS pages automatically use `wss://` for WebSockets. No backend certificates are needed.

### 4. Verify and operate

```sh
curl --fail https://cards.example.org/api/health
docker compose ps
docker compose logs --tail=100 backend frontend
```

Open the HTTPS site, create a room, and join from a separate browser profile. Confirm live updates and reconnect after a refresh. A WebSocket failure usually means `PUBLIC_ORIGIN` does not match the browser origin or nginx is not forwarding upgrades. A 502 usually means an upstream is stopped or its host port does not match nginx. Idle WebSockets may reconnect after the proxy timeout; membership survives.

To deploy a reviewed revision, back up first, then:

```sh
git pull --ff-only
docker compose up -d --build
docker compose ps
```

Rebuilding the backend briefly disconnects players; clients reconnect and persisted games survive compatible updates. `docker compose restart` restarts existing containers but does not apply changed environment variables. `docker compose down` stops and removes containers without deleting the volume. **Do not use `docker compose down -v` unless you intend to delete all stored rooms and replays.**

### Backup and restore

For a consistent filesystem backup, stop the backend and archive the entire volume, including any SQLite WAL files. Run from the same Compose project directory:

```sh
mkdir -p backups
docker compose stop backend
docker compose run --rm --no-deps --user 0:0 \
  -v "$PWD/backups:/backup" --entrypoint tar backend \
  czf /backup/arena-data.tar.gz -C /data .
docker compose start backend
```

Copy the archive off the server and protect access to it. This command overwrites the same archive name; rename it or use dated filenames to retain multiple backups.

Restore into an **empty** data volume using a compatible application revision. With the backend stopped, put the archive in `backups/arena-data.tar.gz`, then run:

```sh
docker compose run --rm --no-deps --user 0:0 \
  -v "$PWD/backups:/backup:ro" --entrypoint tar backend \
  xzf /backup/arena-data.tar.gz -C /data
docker compose up -d
```

The archive preserves the backend user's numeric ownership (UID/GID 10001). Do not extract over a live database or mix a backup with existing database/WAL files.

## Hosting without containers

With stable Rust and Node 22.12+ installed, run from the repository root:

```sh
npm --prefix frontend ci
npm --prefix frontend run build
PUBLIC_ORIGIN=https://cards.example.org BIND_ADDR=127.0.0.1:3000 cargo run --release -p arena-server
```

The Rust server serves `frontend/dist` with SPA routing as well as `/api`. Use a process supervisor for unattended operation. Point both nginx locations at `127.0.0.1:3000` and keep the same WebSocket/TLS configuration. Alternatively, serve the SPA separately and proxy `/api` to Rust.

Outside Compose, backend defaults are `DATABASE_URL=sqlite://arena.db`, `BIND_ADDR=127.0.0.1:3000`, and `PUBLIC_ORIGIN=http://localhost:5173`; `RUST_LOG` controls logging. For LAN development, use `PUBLIC_ORIGIN=http://<your-ip>:5173` and expose the development frontend only on a trusted network.
