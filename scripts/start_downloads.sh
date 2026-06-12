#!/bin/bash
# Serve /opt/apps/clipvault-site on 127.0.0.1:8765.
# Caddy routes /downloads/* and / to this port, so the directory layout
# on disk must mirror the URL paths:
#   /opt/apps/clipvault-site/downloads/ClipVault_0.1.0_x64-setup.exe
#   /opt/apps/clipvault-site/landing/...                 (landing)
pkill -f "http.server 8765" 2>/dev/null
sleep 1
cd /opt/apps/clipvault-site
nohup python3 -m http.server 8765 --bind 127.0.0.1 > /tmp/downloads.log 2>&1 &
sleep 2
echo "PID: $!"
ss -tln | grep 8765
curl -sI http://127.0.0.1:8765/downloads/ClipVault_0.1.0_x64-setup.exe | head -3
