import { useEffect } from "react";
import { useTheme } from "@/stores/theme";

/**
 * Mirrors the persisted theme into the `<html>` class on the main window.
 * The palette window also goes through the same provider; this hook is
 * mostly a placeholder for future integrations (e.g. broadcast across windows).
 */
export function useThemeSync() {
  const { theme } = useTheme();
  useEffect(() => {
    // ThemeProvider already applies the class. This hook exists so a future
    // requirement (e.g. notify other windows) can be added without touching call sites.
  }, [theme]);
}
