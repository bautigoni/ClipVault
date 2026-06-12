#!/usr/bin/env python3
"""Replace the clipvault.bauhub.online block in /etc/caddy/Caddyfile with
a version that routes both / and /downloads/* to the static Python
file server (127.0.0.1:8765), which serves the landing dist + the
installer from /opt/apps/clipvault-site."""
PATH = "/etc/caddy/Caddyfile"
with open(PATH) as f:
    s = f.read()
key = "clipvault.bauhub.online {"
start = s.find(key)
assert start >= 0, "no clipvault block"
depth = 0
i = s.find("{", start)
end = i
while i < len(s):
    if s[i] == "{":
        depth += 1
    elif s[i] == "}":
        depth -= 1
        if depth == 0:
            end = i + 1
            break
    i += 1
while end < len(s) and s[end] in "\n\r ":
    end += 1
new_block = (
    "clipvault.bauhub.online {\n"
    "    reverse_proxy 127.0.0.1:8765\n"
    "}\n"
)
new = s[:start] + new_block + s[end:]
with open(PATH, "w") as f:
    f.write(new)
print("Wrote fixed Caddyfile.")
