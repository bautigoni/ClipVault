# Lightweight static file server for the ClipVault landing. The dist/ is
# built locally with `npm run build` and copied into the image so we don't
# need node/npm inside the container.
#
# The `docker-compose.yml` runs from the parent dir, so we expect the
# static files to live at `site/` (one level above this Dockerfile).
FROM caddy:2-alpine

# Caddy's default static file config works for us: serve /srv from CWD.
# We override the default Caddyfile to keep it minimal and read-only.
RUN rm -f /etc/caddy/Caddyfile
COPY <<EOF /etc/caddy/Caddyfile
:3000 {
    root * /srv
    encode gzip zstd
    file_server
    # Cache static assets aggressively; the index.html stays dynamic.
    @assets path /assets/*
    header @assets Cache-Control "public, max-age=31536000, immutable"
    try_files {path} {path}.html /index.html
}
EOF

COPY --chown=caddy:caddy site/ /srv/
