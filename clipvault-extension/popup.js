const enabled = document.getElementById("enabled");
const port = document.getElementById("port");

chrome.storage.local.get({ port: 3939, enabled: true }, (s) => {
  enabled.checked = s.enabled;
  port.value = s.port;
});

enabled.addEventListener("change", () => {
  chrome.storage.local.set({ enabled: enabled.checked });
});
port.addEventListener("change", () => {
  chrome.storage.local.set({ port: Number(port.value) });
});
