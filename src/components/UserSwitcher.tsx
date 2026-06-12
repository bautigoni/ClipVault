import { useEffect, useRef, useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { ChevronDown, Plus, Trash2, User as UserIcon } from "lucide-react";
import { api, type User } from "@/lib/tauri";

/**
 * Top-right user switcher. Lists every user in the DB, lets the user
 * switch active profile, create a new profile, rename, set-as-default,
 * or delete (with safety checks). The active user id is persisted in
 * the `active_user_id` setting on the Rust side.
 */
export function UserSwitcher() {
  const qc = useQueryClient();
  const [open, setOpen] = useState(false);
  const [newOpen, setNewOpen] = useState(false);
  const [newName, setNewName] = useState("");
  const [renamingId, setRenamingId] = useState<string | null>(null);
  const [renameDraft, setRenameDraft] = useState("");
  const ref = useRef<HTMLDivElement>(null);

  const usersQ = useQuery({
    queryKey: ["users"],
    queryFn: api.usersList,
  });
  const activeQ = useQuery({
    queryKey: ["active-user"],
    queryFn: api.usersGetActive,
  });

  useEffect(() => {
    if (!open) return;
    const onDown = (e: MouseEvent) => {
      if (!ref.current?.contains(e.target as Node)) {
        setOpen(false);
        setNewOpen(false);
        setRenamingId(null);
      }
    };
    window.addEventListener("mousedown", onDown);
    return () => window.removeEventListener("mousedown", onDown);
  }, [open]);

  const switchTo = async (id: string) => {
    try {
      await api.usersSetActive(id);
      qc.invalidateQueries({ queryKey: ["active-user"] });
      qc.invalidateQueries({ queryKey: ["clips"] });
      qc.invalidateQueries({ queryKey: ["palette"] });
      qc.invalidateQueries({ queryKey: ["collections"] });
      qc.invalidateQueries({ queryKey: ["snippets"] });
      qc.invalidateQueries({ queryKey: ["activity"] });
      setOpen(false);
    } catch (e) {
      console.error("usersSetActive failed", e);
      alert(`Couldn't switch user: ${(e as Error).message ?? e}`);
    }
  };

  const createUser = async () => {
    const trimmed = newName.trim();
    if (!trimmed) return;
    try {
      const u = await api.usersCreate(trimmed, null);
      await api.usersSetActive(u.id);
      setNewName("");
      setNewOpen(false);
      qc.invalidateQueries({ queryKey: ["users"] });
      qc.invalidateQueries({ queryKey: ["active-user"] });
    } catch (e) {
      console.error("usersCreate failed", e);
      alert(`Couldn't create user: ${(e as Error).message ?? e}`);
    }
  };

  const rename = async (id: string) => {
    const trimmed = renameDraft.trim();
    if (!trimmed) return;
    try {
      await api.usersRename(id, trimmed);
      qc.invalidateQueries({ queryKey: ["users"] });
      qc.invalidateQueries({ queryKey: ["active-user"] });
      setRenamingId(null);
    } catch (e) {
      console.error("usersRename failed", e);
      alert(`Couldn't rename: ${(e as Error).message ?? e}`);
    }
  };

  const setDefault = async (id: string) => {
    try {
      await api.usersSetDefault(id);
      qc.invalidateQueries({ queryKey: ["users"] });
    } catch (e) {
      console.error("usersSetDefault failed", e);
      alert(`Couldn't set default: ${(e as Error).message ?? e}`);
    }
  };

  const remove = async (id: string) => {
    if (!confirm("Delete this user? Their clips will be kept but unassigned.")) return;
    try {
      await api.usersDelete(id);
      qc.invalidateQueries({ queryKey: ["users"] });
      qc.invalidateQueries({ queryKey: ["active-user"] });
    } catch (e) {
      console.error("usersDelete failed", e);
      alert(`Couldn't delete: ${(e as Error).message ?? e}`);
    }
  };

  const active = activeQ.data;
  const users = usersQ.data ?? [];
  const initials = (active?.display_name ?? "?").slice(0, 1).toUpperCase();

  return (
    <div className="relative" ref={ref}>
      <button
        type="button"
        title="Switch profile"
        onClick={() => setOpen((v) => !v)}
        className="flex items-center gap-2 rounded-md border border-border bg-bg px-2 py-1 text-sm text-fg transition-colors hover:bg-bg-overlay"
      >
        <span className="grid h-6 w-6 place-items-center rounded-full bg-accent text-[11px] font-bold text-accent-fg">
          {initials}
        </span>
        <span className="max-w-[120px] truncate">{active?.display_name ?? "—"}</span>
        <ChevronDown className="h-3.5 w-3.5 text-fg-muted" />
      </button>
      {open && (
        <div className="absolute right-0 top-full z-50 mt-1 w-72 overflow-hidden rounded-md border border-border bg-bg-elevated shadow-xl">
          <div className="px-3 py-2 text-[10px] font-semibold uppercase tracking-wide text-fg-muted">
            Profiles
          </div>
          <ul className="max-h-64 overflow-auto">
            {users.map((u) => (
              <UserRow
                key={u.id}
                user={u}
                isActive={u.id === active?.id}
                isRenaming={renamingId === u.id}
                renameDraft={renameDraft}
                onStartRename={() => {
                  setRenamingId(u.id);
                  setRenameDraft(u.display_name);
                }}
                onRenameDraftChange={setRenameDraft}
                onRename={() => rename(u.id)}
                onCancelRename={() => setRenamingId(null)}
                onSwitch={() => switchTo(u.id)}
                onSetDefault={() => setDefault(u.id)}
                onDelete={() => remove(u.id)}
              />
            ))}
            {users.length === 0 && (
              <li className="px-3 py-2 text-xs text-fg-muted">No profiles yet</li>
            )}
          </ul>
          <div className="border-t border-border">
            {newOpen ? (
              <div className="flex items-center gap-1 px-2 py-1.5">
                <input
                  type="text"
                  autoFocus
                  value={newName}
                  onChange={(e) => setNewName(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") void createUser();
                    if (e.key === "Escape") {
                      setNewOpen(false);
                      setNewName("");
                    }
                  }}
                  placeholder="Profile name"
                  className="flex-1 rounded border border-border bg-bg px-2 py-1 text-xs text-fg outline-none focus:border-accent"
                />
                <button
                  type="button"
                  onClick={createUser}
                  className="rounded bg-accent px-2 py-1 text-[11px] font-semibold text-accent-fg hover:bg-accent/90"
                >
                  Add
                </button>
              </div>
            ) : (
              <button
                type="button"
                onClick={() => setNewOpen(true)}
                className="flex w-full items-center gap-2 px-3 py-2 text-xs text-fg-muted hover:bg-bg-overlay hover:text-fg"
              >
                <Plus className="h-3.5 w-3.5" />
                New profile
              </button>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

function UserRow({
  user,
  isActive,
  isRenaming,
  renameDraft,
  onStartRename,
  onRenameDraftChange,
  onRename,
  onCancelRename,
  onSwitch,
  onSetDefault,
  onDelete,
}: {
  user: User;
  isActive: boolean;
  isRenaming: boolean;
  renameDraft: string;
  onStartRename: () => void;
  onRenameDraftChange: (v: string) => void;
  onRename: () => void;
  onCancelRename: () => void;
  onSwitch: () => void;
  onSetDefault: () => void;
  onDelete: () => void;
}) {
  return (
    <li
      className={`group flex items-center gap-2 px-2 py-1.5 hover:bg-bg-overlay ${
        isActive ? "bg-accent/10" : ""
      }`}
    >
      <span className="grid h-6 w-6 shrink-0 place-items-center rounded-full bg-bg-overlay text-[11px] font-bold text-fg-muted">
        {user.display_name.slice(0, 1).toUpperCase()}
      </span>
      {isRenaming ? (
        <input
          type="text"
          autoFocus
          value={renameDraft}
          onChange={(e) => onRenameDraftChange(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") onRename();
            if (e.key === "Escape") onCancelRename();
          }}
          onBlur={onCancelRename}
          className="flex-1 rounded border border-border bg-bg px-1 py-0.5 text-xs text-fg outline-none focus:border-accent"
        />
      ) : (
        <button
          type="button"
          onClick={onSwitch}
          onDoubleClick={onStartRename}
          className="flex-1 truncate text-left text-xs text-fg"
        >
          {user.display_name}
          {user.is_default && (
            <span className="ml-1 rounded bg-bg-overlay px-1 text-[9px] uppercase text-fg-muted">
              default
            </span>
          )}
        </button>
      )}
      <button
        type="button"
        title="Rename"
        onClick={onStartRename}
        className="rounded p-1 text-fg-muted opacity-0 transition-opacity hover:bg-bg-overlay hover:text-fg group-hover:opacity-100"
      >
        <UserIcon className="h-3 w-3" />
      </button>
      {!user.is_default && (
        <button
          type="button"
          title="Set as default"
          onClick={onSetDefault}
          className="rounded p-1 text-fg-muted opacity-0 transition-opacity hover:bg-bg-overlay hover:text-fg group-hover:opacity-100"
        >
          <span className="text-[10px]">★</span>
        </button>
      )}
      {!user.is_default && (
        <button
          type="button"
          title="Delete"
          onClick={onDelete}
          className="rounded p-1 text-fg-muted opacity-0 transition-opacity hover:bg-bg-overlay hover:text-error group-hover:opacity-100"
        >
          <Trash2 className="h-3 w-3" />
        </button>
      )}
    </li>
  );
}
