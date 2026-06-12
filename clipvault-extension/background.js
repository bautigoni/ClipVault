// ClipVault browser bridge. Sends selected text and page metadata to the local
// ClipVault HTTP receiver, which is opt-in and bound to 127.0.0.1 by default.

const DEFAULT_PORT = 3939; // must match ClipVault's http_receiver port

async function getSettings() {
  return new Promise((resolve) => {
    chrome.storage.local.get({ port: DEFAULT_PORT, enabled: true }, (s) => resolve(s));
  });
}

async function postToClipVault(path, body) {
  const { port, enabled } = await getSettings();
  if (!enabled) return { ok: false, reason: "disabled" };
  try {
    const res = await fetch(`http://127.0.0.1:${port}${path}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });
    return { ok: res.ok };
  } catch (e) {
    return { ok: false, reason: String(e) };
  }
}

chrome.runtime.onInstalled.addListener(() => {
  chrome.contextMenus.create({
    id: "clipvault-send-selection",
    title: "Send selection to ClipVault",
    contexts: ["selection"],
  });
});

chrome.contextMenus.onClicked.addListener(async (info, tab) => {
  if (info.menuItemId !== "clipvault-send-selection") return;
  if (!info.selectionText) return;
  const res = await postToClipVault("/clips", {
    type: "text",
    text: info.selectionText,
    source_app: tab?.url ? new URL(tab.url).hostname : null,
    source_title: tab?.title ?? null,
    captured_at: Date.now(),
  });
  if (!res.ok) {
    console.warn("[clipvault] send failed", res);
  }
});
