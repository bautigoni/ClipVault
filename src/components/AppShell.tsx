import { Link, useLocation } from "react-router-dom";
import {
  Clock,
  Star,
  Folder,
  Code2,
  Image as ImageIcon,
  Settings as SettingsIcon,
  Search,
} from "lucide-react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useState, type ReactNode } from "react";
import { api } from "@/lib/tauri";
import { cn } from "@/lib/utils";

const navItems = [
  { to: "/timeline", label: "Timeline", icon: Clock },
  { to: "/favorites", label: "Favorites", icon: Star },
  { to: "/collections", label: "Collections", icon: Folder },
  { to: "/snippets", label: "Snippets", icon: Code2 },
  { to: "/images", label: "Images", icon: ImageIcon },
];

export function AppShell({ children }: { children: ReactNode }) {
  const location = useLocation();
  const qc = useQueryClient();
  const collections = useQuery({
    queryKey: ["collections"],
    queryFn: api.listCollections,
  });
  const [dropTargetId, setDropTargetId] = useState<string | null>(null);

  const handleDrop = async (e: React.DragEvent, collectionId: string) => {
    e.preventDefault();
    setDropTargetId(null);
    const clipId = e.dataTransfer.getData("application/x-clipvault-clip");
    if (!clipId) return;
    try {
      await api.assignToCollection(clipId, collectionId);
      qc.invalidateQueries({ queryKey: ["clips"] });
      qc.invalidateQueries({ queryKey: ["collections"] });
    } catch (err) {
      console.error("Failed to assign to collection", err);
    }
  };

  return (
    <div className="flex h-full">
      <aside className="flex w-60 shrink-0 flex-col border-r border-border bg-bg-elevated">
        <div className="flex items-center gap-2 px-4 py-4">
          <div className="grid h-8 w-8 place-items-center rounded-md bg-accent text-accent-fg font-bold">
            CV
          </div>
          <div>
            <h1 className="text-sm font-semibold leading-none">ClipVault</h1>
            <p className="text-[11px] text-fg-muted">local clipboard</p>
          </div>
        </div>
        <div className="px-2">
          <button
            onClick={() => api.showPalette()}
            className="flex w-full items-center gap-2 rounded-md border border-border bg-bg px-3 py-1.5 text-sm text-fg-muted transition-colors hover:bg-bg-overlay hover:text-fg"
          >
            <Search className="h-4 w-4" />
            <span>Quick search</span>
            <span className="ml-auto kbd">Ctrl+Shift+V</span>
          </button>
        </div>
        <nav className="mt-4 flex flex-col gap-0.5 px-2">
          {navItems.map((item) => {
            const Icon = item.icon;
            const active = location.pathname.startsWith(item.to);
            return (
              <Link
                key={item.to}
                to={item.to}
                className={cn("sidebar-item", active && "bg-accent/15 text-fg")}
              >
                <span className="flex items-center gap-2">
                  <Icon className="h-4 w-4" />
                  {item.label}
                </span>
              </Link>
            );
          })}
        </nav>
        <div className="mt-4 px-3 text-[10px] uppercase tracking-wide text-fg-subtle">
          Collections
        </div>
        <nav className="flex flex-col gap-0.5 px-2">
          <Link
            to="/timeline"
            data-drop-target={dropTargetId === "__unfiled__"}
            className={cn(
              "sidebar-item italic",
              dropTargetId === "__unfiled__" && "ring-2 ring-accent bg-accent/10"
            )}
            onDragOver={(e) => {
              if (e.dataTransfer.types.includes("application/x-clipvault-clip")) {
                e.preventDefault();
                e.dataTransfer.dropEffect = "move";
                setDropTargetId("__unfiled__");
              }
            }}
            onDragLeave={() => {
              if (dropTargetId === "__unfiled__") setDropTargetId(null);
            }}
            onDrop={async (e) => {
              e.preventDefault();
              setDropTargetId(null);
              const clipId = e.dataTransfer.getData("application/x-clipvault-clip");
              if (!clipId) return;
              try {
                await api.assignToCollection(clipId, null);
                qc.invalidateQueries({ queryKey: ["clips"] });
                qc.invalidateQueries({ queryKey: ["collections"] });
              } catch (err) {
                console.error("Failed to unassign", err);
              }
            }}
          >
            <span className="truncate">Unfiled</span>
          </Link>
          {collections.data?.map((c) => {
            const active = location.pathname === `/collections/${c.id}`;
            return (
              <Link
                key={c.id}
                to={`/collections/${c.id}`}
                data-active={active}
                data-drop-target={dropTargetId === c.id}
                className={cn(
                  "sidebar-item",
                  dropTargetId === c.id && "ring-2 ring-accent bg-accent/10"
                )}
                onDragOver={(e) => {
                  if (e.dataTransfer.types.includes("application/x-clipvault-clip")) {
                    e.preventDefault();
                    e.dataTransfer.dropEffect = "move";
                    setDropTargetId(c.id);
                  }
                }}
                onDragLeave={() => {
                  if (dropTargetId === c.id) setDropTargetId(null);
                }}
                onDrop={(e) => handleDrop(e, c.id)}
              >
                <span className="flex items-center gap-2 truncate">
                  <Folder className="h-4 w-4" />
                  {c.name}
                </span>
                <span className="ml-auto text-[11px] text-fg-subtle">{c.clip_count}</span>
              </Link>
            );
          })}
        </nav>
        <div className="mt-auto border-t border-border p-2">
          <Link
            to="/settings"
            className={cn(
              "sidebar-item",
              location.pathname === "/settings" && "bg-accent/15 text-fg"
            )}
          >
            <span className="flex items-center gap-2">
              <SettingsIcon className="h-4 w-4" />
              Settings
            </span>
          </Link>
        </div>
      </aside>
      <main className="flex-1 overflow-hidden">{children}</main>
    </div>
  );
}
