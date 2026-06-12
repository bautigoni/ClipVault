import { Routes, Route, Navigate } from "react-router-dom";
import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { AppShell } from "./components/AppShell";
import { TimelinePage } from "./routes/TimelinePage";
import { FavoritesPage } from "./routes/FavoritesPage";
import { CollectionsPage } from "./routes/CollectionsPage";
import { SnippetsPage } from "./routes/SnippetsPage";
import { SettingsPage } from "./routes/SettingsPage";
import { ImagesPage } from "./routes/ImagesPage";
import { useClipsInvalidation } from "./hooks/useClipsInvalidation";
import { useThemeSync } from "./hooks/useThemeSync";

export default function App() {
  useClipsInvalidation();
  useThemeSync();
  useEffect(() => {
    let cancelled = false;
    let unlistenFn: (() => void) | null = null;
    listen("clip://ready", () => {
      // Could trigger a re-fetch here. Hooks handle this.
    })
      .then((fn) => {
        if (cancelled) {
          // Component already unmounted; detach immediately.
          fn();
        } else {
          unlistenFn = fn;
        }
      })
      .catch((err) => {
        console.error("clip://ready listen failed", err);
      });
    return () => {
      cancelled = true;
      if (unlistenFn) unlistenFn();
    };
  }, []);

  return (
    <AppShell>
      <Routes>
        <Route path="/" element={<Navigate to="/timeline" replace />} />
        <Route path="/timeline" element={<TimelinePage />} />
        <Route path="/favorites" element={<FavoritesPage />} />
        <Route path="/collections" element={<CollectionsPage />} />
        <Route path="/collections/:id" element={<CollectionsPage />} />
        <Route path="/snippets" element={<SnippetsPage />} />
        <Route path="/images" element={<ImagesPage />} />
        <Route path="/settings" element={<SettingsPage />} />
        <Route path="*" element={<Navigate to="/timeline" replace />} />
      </Routes>
    </AppShell>
  );
}
