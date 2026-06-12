import { useEffect, useRef, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Save as SaveIcon, FolderOpen, Download, Upload, RotateCw, AlertCircle } from "lucide-react";
import { open, save as saveDialog } from "@tauri-apps/plugin-dialog";
import { api } from "@/lib/tauri";
import { useTheme } from "@/stores/theme";
import type { Settings } from "@/types";
import { settingsSchema, type SettingsInput } from "@/lib/schemas";
import { HotkeyRecorder } from "@/components/HotkeyRecorder";

const defaultSettings: Settings = {
  retention_days: 0,
  max_clips: 1_000_000,
  hotkey: "CommandOrControl+Shift+V",
  theme: "system",
  autostart: false,
  storage_dir: null,
  excluded_apps: [],
  sensitive_apps: [],
  auto_paste: true,
  backup_enabled: false,
  backup_dir: null,
  local_only: true,
  sync_endpoint: null,
  http_receiver_enabled: false,
  ring_hotkey_reverse: "CommandOrControl+Shift+V",
  ring_hotkey_forward: "CommandOrControl+Shift+Alt+V",
  ring_hotkey_overlay: "CommandOrControl+Shift+R",
  ring_capacity: 64,
  ring_idle_dismiss_ms: 30_000,
  ring_wrap: true,
  ring_include_sensitive: false,
  ring_include_files: true,
  ring_include_images: true,
  merge_separator: " ",
  palette_jump_size: 0,
};

export function SettingsPage() {
  const qc = useQueryClient();
  const { theme, setTheme } = useTheme();
  const settings = useQuery({ queryKey: ["settings"], queryFn: api.getSettings });
  const [draft, setDraft] = useState<Settings>(defaultSettings);
  const [errors, setErrors] = useState<Record<string, string>>({});

  // Only seed the draft from the server once. After that, user edits own the
  // draft state and we re-sync only on a successful save (handled in onSuccess).
  const seededRef = useRef(false);
  useEffect(() => {
    if (settings.data && !seededRef.current) {
      setDraft(settings.data);
      seededRef.current = true;
    }
  }, [settings.data]);

  const save = useMutation({
    mutationFn: () => {
      const parsed = settingsSchema.safeParse(draft satisfies SettingsInput);
      if (!parsed.success) {
        const map: Record<string, string> = {};
        for (const issue of parsed.error.issues) {
          const key = issue.path.join(".");
          if (!map[key]) map[key] = issue.message;
        }
        setErrors(map);
        throw new Error("Invalid settings");
      }
      setErrors({});
      return api.updateSettings(draft);
    },
    onSuccess: async (s) => {
      qc.setQueryData(["settings"], s);
      setDraft(s);
      setErrors({});
      try {
        if (s.hotkey) {
          await api.registerHotkey(s.hotkey);
        }
      } catch (err) {
        console.error("Failed to register hotkey", err);
      }
    },
    onError: (err) => {
      console.error("Failed to save settings", err);
    },
  });

  return (
    <div className="flex h-full flex-col">
      <header className="flex items-center gap-2 border-b border-border px-6 py-3">
        <h2 className="text-lg font-semibold">Settings</h2>
      </header>
      <div className="flex-1 overflow-y-auto p-6">
        <div className="mx-auto flex max-w-3xl flex-col gap-6">
          <Section title="Appearance">
            <Row label="Theme">
              <select
                value={theme}
                onChange={(e) => setTheme(e.target.value as Settings["theme"])}
                className="input max-w-[180px]"
              >
                <option value="system">System</option>
                <option value="light">Light</option>
                <option value="dark">Dark</option>
                <option value="graphite">Graphite</option>
              </select>
            </Row>
          </Section>

          <Section title="Behavior">
            <Row label="Hotkey">
              <HotkeyRecorder
                value={draft.hotkey}
                onChange={(combo) => setDraft({ ...draft, hotkey: combo })}
              />
              <FieldError message={errors.hotkey} />
            </Row>
            <Row label="Auto-paste after picking">
              <Toggle
                value={draft.auto_paste}
                onChange={(v) => setDraft({ ...draft, auto_paste: v })}
              />
              <p className="mt-1 text-[11px] text-fg-muted">
                When on, picking a clip (Enter / click) pastes it into the previously focused app
                automatically. On by default — turn off if you'd rather press Ctrl+V yourself.
              </p>
            </Row>
            <Row label="Merge separator">
              <input
                type="text"
                value={draft.merge_separator}
                onChange={(e) => setDraft({ ...draft, merge_separator: e.target.value })}
                placeholder='e.g. " ", "\n", "" (none)'
                className="w-32 rounded-md border border-border bg-bg-overlay px-2 py-1 text-sm text-fg focus:border-accent focus:outline-none"
              />
              <p className="mt-1 text-[11px] text-fg-muted">
                String inserted between clips when you multi-select and press Enter. Default: a
                single space, so "admin" + "alar la v" → "admin alar la v". Use "\n" for newlines
                or "" to concatenate with no separator.
              </p>
            </Row>
            <Row label="Ctrl + ↑↓ jump size">
              <input
                type="number"
                min={0}
                max={1000}
                value={draft.palette_jump_size}
                onChange={(e) => {
                  const n = Math.max(0, Math.min(1000, Number(e.target.value) || 0));
                  setDraft({ ...draft, palette_jump_size: n });
                }}
                className="w-24 rounded-md border border-border bg-bg-overlay px-2 py-1 text-sm text-fg focus:border-accent focus:outline-none"
              />
              <p className="mt-1 text-[11px] text-fg-muted">
                How many rows to jump when holding Ctrl + ↑/↓ in the palette. Default: 0, which
                snaps to the very top / very bottom — useful for long histories.
              </p>
            </Row>
            <Row label="Start with Windows">
              <Toggle
                value={draft.autostart}
                onChange={(v) => setDraft({ ...draft, autostart: v })}
              />
            </Row>
          </Section>

          <Section title="Clipboard Ring">
            <p className="-mt-2 mb-2 text-xs text-fg-muted">
              Cycle through recent clips with a hotkey, then paste with Ctrl+V.
              The ring is dismissed when you copy something new.
            </p>
            <Row label="Reverse (older)">
              <HotkeyRecorder
                value={draft.ring_hotkey_reverse}
                onChange={(combo) =>
                  setDraft({ ...draft, ring_hotkey_reverse: combo })
                }
              />
              <FieldError message={errors.ring_hotkey_reverse} />
            </Row>
            <Row label="Forward (newer)">
              <HotkeyRecorder
                value={draft.ring_hotkey_forward}
                onChange={(combo) =>
                  setDraft({ ...draft, ring_hotkey_forward: combo })
                }
              />
              <FieldError message={errors.ring_hotkey_forward} />
            </Row>
            <Row label="Show overlay">
              <HotkeyRecorder
                value={draft.ring_hotkey_overlay}
                onChange={(combo) =>
                  setDraft({ ...draft, ring_hotkey_overlay: combo })
                }
              />
              <FieldError message={errors.ring_hotkey_overlay} />
            </Row>
            <Row label="Capacity">
              <input
                type="number"
                min={1}
                max={1024}
                value={draft.ring_capacity}
                onChange={(e) =>
                  setDraft({ ...draft, ring_capacity: Number(e.target.value) })
                }
                className={`input max-w-[120px] ${errors.ring_capacity ? "border-danger" : ""}`}
                aria-invalid={!!errors.ring_capacity}
              />
              <span className="text-xs text-fg-muted">slots (1–1024)</span>
              <FieldError message={errors.ring_capacity} />
            </Row>
            <Row label="Idle dismiss (ms)">
              <input
                type="number"
                min={0}
                step={1000}
                value={draft.ring_idle_dismiss_ms}
                onChange={(e) =>
                  setDraft({ ...draft, ring_idle_dismiss_ms: Number(e.target.value) })
                }
                className={`input max-w-[120px] ${errors.ring_idle_dismiss_ms ? "border-danger" : ""}`}
                aria-invalid={!!errors.ring_idle_dismiss_ms}
              />
              <span className="text-xs text-fg-muted">0 = no auto-dismiss</span>
              <FieldError message={errors.ring_idle_dismiss_ms} />
            </Row>
            <Row label="Wrap around">
              <Toggle
                value={draft.ring_wrap}
                onChange={(v) => setDraft({ ...draft, ring_wrap: v })}
              />
            </Row>
            <Row label="Include sensitive clips">
              <Toggle
                value={draft.ring_include_sensitive}
                onChange={(v) => setDraft({ ...draft, ring_include_sensitive: v })}
              />
            </Row>
            <Row label="Include file clips">
              <Toggle
                value={draft.ring_include_files}
                onChange={(v) => setDraft({ ...draft, ring_include_files: v })}
              />
            </Row>
            <Row label="Include image clips">
              <Toggle
                value={draft.ring_include_images}
                onChange={(v) => setDraft({ ...draft, ring_include_images: v })}
              />
            </Row>
          </Section>

          <Section title="Storage">
            <Row label="Retention (days)">
              <input
                type="number"
                min={0}
                value={draft.retention_days}
                onChange={(e) =>
                  setDraft({ ...draft, retention_days: Number(e.target.value) })
                }
                className={`input max-w-[120px] ${errors.retention_days ? "border-danger" : ""}`}
                aria-invalid={!!errors.retention_days}
              />
              <span className="text-xs text-fg-muted">0 = infinite</span>
              <FieldError message={errors.retention_days} />
            </Row>
            <Row label="Max clips">
              <input
                type="number"
                min={1000}
                step={1000}
                value={draft.max_clips}
                onChange={(e) => setDraft({ ...draft, max_clips: Number(e.target.value) })}
                className={`input max-w-[160px] ${errors.max_clips ? "border-danger" : ""}`}
                aria-invalid={!!errors.max_clips}
              />
              <FieldError message={errors.max_clips} />
            </Row>
            <Row label="Excluded apps (one per line)">
              <textarea
                value={draft.excluded_apps.join("\n")}
                onChange={(e) =>
                  setDraft({
                    ...draft,
                    excluded_apps: e.target.value.split("\n").map((s) => s.trim()).filter(Boolean),
                  })
                }
                className="input min-h-[80px] font-mono text-xs"
              />
            </Row>
          </Section>

          <Section title="Backup">
            <Row label="Enable scheduled backups">
              <Toggle
                value={draft.backup_enabled}
                onChange={(v) => setDraft({ ...draft, backup_enabled: v })}
              />
            </Row>
            <Row label="Backup folder">
              <div className="flex flex-1 items-center gap-2">
                <input
                  value={draft.backup_dir ?? ""}
                  onChange={(e) => setDraft({ ...draft, backup_dir: e.target.value || null })}
                  className="input flex-1"
                  placeholder="leave empty for default"
                />
                <button
                  className="btn"
                  onClick={async () => {
                    const dir = await open({ directory: true });
                    if (typeof dir === "string") setDraft({ ...draft, backup_dir: dir });
                  }}
                >
                  <FolderOpen className="h-4 w-4" /> Browse
                </button>
                <button
                  className="btn"
                  onClick={async () => {
                    await api.runBackup();
                    alert("Backup written to " + (draft.backup_dir ?? "default folder"));
                  }}
                >
                  <RotateCw className="h-4 w-4" /> Run now
                </button>
              </div>
            </Row>
          </Section>

          <Section title="Import / Export">
            <Row label="Database">
              <div className="flex gap-2">
                <button
                  className="btn"
                  onClick={async () => {
                    const path = await saveDialog({
                      defaultPath: "clipvault-export.clipvault",
                      filters: [{ name: "ClipVault", extensions: ["clipvault"] }],
                    });
                    if (path) {
                      await api.exportDb(path);
                      alert("Export complete.");
                    }
                  }}
                >
                  <Download className="h-4 w-4" /> Export
                </button>
                <button
                  className="btn"
                  onClick={async () => {
                    const path = await open({
                      multiple: false,
                      filters: [{ name: "ClipVault", extensions: ["clipvault"] }],
                    });
                    if (typeof path === "string") {
                      const report = await api.importDb(path, "duplicate");
                      alert(
                        `Imported ${report.clips_added} clips. ${report.errors.length} errors.`
                      );
                    }
                  }}
                >
                  <Upload className="h-4 w-4" /> Import
                </button>
              </div>
            </Row>
          </Section>

          <Section title="Privacy">
            <p className="text-sm text-fg-muted">
              ClipVault runs entirely on your device. No cloud, no telemetry, no analytics. The
              only network calls are the optional self-hosted sync (off by default) and the
              browser extension receiver (off by default).
            </p>
            <Row label="Modo local (recomendado)">
              <Toggle
                value={draft.local_only}
                onChange={(v) => {
                  // Going from local -> cloud: require an explicit confirmation.
                  if (draft.local_only && !v) {
                    const ok = window.confirm(
                      "¿Entendés que esto va a hacer requests a un endpoint que vos configures?"
                    );
                    if (!ok) return;
                  }
                  // Going from cloud -> local: clear the cloud fields so the
                  // backend guard does not have to drop them silently.
                  setDraft({
                    ...draft,
                    local_only: v,
                    ...(v
                      ? { sync_endpoint: null, http_receiver_enabled: false }
                      : {}),
                  });
                }}
              />
              {draft.local_only ? (
                <span
                  className="ml-2 inline-flex items-center gap-1 rounded-md border border-success/30 bg-success/10 px-2 py-0.5 text-xs text-success"
                  title="No traffic leaves this device."
                >
                  No se envían datos fuera de tu PC
                </span>
              ) : (
                <span className="ml-2 text-xs text-fg-muted">
                  Sync y receptor habilitados — tráfico de red permitido.
                </span>
              )}
            </Row>
            <Row label="Sync endpoint">
              <input
                value={draft.sync_endpoint ?? ""}
                disabled={draft.local_only}
                onChange={(e) =>
                  setDraft({
                    ...draft,
                    sync_endpoint: e.target.value || null,
                  })
                }
                className="input flex-1 disabled:cursor-not-allowed disabled:opacity-50"
                placeholder={draft.local_only ? "Deshabilitado en modo local" : "https://example.com/clipvault-sync"}
              />
              <FieldError message={errors.sync_endpoint} />
            </Row>
            <Row label="HTTP receiver (browser ext.)">
              <Toggle
                value={draft.http_receiver_enabled}
                onChange={(v) => setDraft({ ...draft, http_receiver_enabled: v })}
              />
              {draft.local_only && (
                <span className="ml-2 text-xs text-fg-muted">
                  Forzado a OFF mientras Modo local esté activo.
                </span>
              )}
            </Row>
          </Section>

          <Section title="Danger zone">
            <button
              className="btn text-danger"
              onClick={async () => {
                if (confirm("Clear all non-favorite, non-pinned clips? This cannot be undone.")) {
                  const count = await api.clearHistory();
                  alert(`Cleared ${count} clips.`);
                }
              }}
            >
              Clear history
            </button>
          </Section>

          <div className="sticky bottom-0 -mx-6 mt-2 flex items-center justify-end gap-2 border-t border-border bg-bg/80 px-6 py-3 backdrop-blur">
            <button
              className="btn-primary"
              disabled={save.isPending}
              onClick={() => save.mutate()}
            >
              <SaveIcon className="h-4 w-4" />
              {save.isPending ? "Saving…" : "Save settings"}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="rounded-lg border border-border bg-bg-elevated p-4">
      <h3 className="mb-3 text-sm font-semibold uppercase tracking-wide text-fg-muted">
        {title}
      </h3>
      <div className="flex flex-col gap-3">{children}</div>
    </section>
  );
}

function Row({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex items-center gap-3">
      <span className="w-40 text-sm text-fg-muted">{label}</span>
      <div className="flex flex-1 items-center gap-2">{children}</div>
    </div>
  );
}

function Toggle({ value, onChange }: { value: boolean; onChange: (v: boolean) => void }) {
  return (
    <button
      role="switch"
      aria-checked={value}
      onClick={() => onChange(!value)}
      className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
        value ? "bg-accent" : "bg-bg-overlay"
      }`}
    >
      <span
        className={`inline-block h-5 w-5 transform rounded-full bg-white transition-transform ${
          value ? "translate-x-5" : "translate-x-0.5"
        }`}
      />
    </button>
  );
}

function FieldError({ message }: { message?: string }) {
  if (!message) return null;
  return (
    <span className="ml-2 inline-flex items-center gap-1 text-xs text-danger">
      <AlertCircle className="h-3 w-3" />
      {message}
    </span>
  );
}
