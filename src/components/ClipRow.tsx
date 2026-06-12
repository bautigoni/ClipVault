import { useEffect, useState } from "react";
import {
  FileText,
  Image as ImageIcon,
  Files,
  Link2,
  Star,
  Pin,
  MoreHorizontal,
  Copy as CopyIcon,
  Trash2,
  Heart,
} from "lucide-react";
import { useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/tauri";
import { cn, formatBytes, formatTimeAgo } from "@/lib/utils";
import type { Clip } from "@/types";

const TYPE_ICONS = {
  text: FileText,
  image: ImageIcon,
  files: Files,
  url: Link2,
} as const;

interface Props {
  clip: Clip;
  onSelect?: (clip: Clip) => void;
  showCollection?: boolean;
  draggable?: boolean;
}

export function ClipRow({ clip, onSelect, showCollection, draggable = true }: Props) {
  const Icon = TYPE_ICONS[clip.type];
  const [thumb, setThumb] = useState<string | null>(null);
  const qc = useQueryClient();

  useEffect(() => {
    let active = true;
    let urlToRevoke: string | null = null;
    if (clip.type === "image" && clip.image) {
      api
        .readImageThumb(clip.image.thumb_path)
        .then((bytes) => {
          if (!active) return;
          const arr = bytes instanceof ArrayBuffer ? new Uint8Array(bytes) : new Uint8Array(bytes);
          const blob = new Blob([arr], { type: "image/jpeg" });
          urlToRevoke = URL.createObjectURL(blob);
          setThumb(urlToRevoke);
        })
        .catch(() => {});
    }
    return () => {
      active = false;
      if (urlToRevoke) URL.revokeObjectURL(urlToRevoke);
    };
  }, [clip.id, clip.type, clip.image?.thumb_path]);

  return (
    <div
      className={cn(
        "clip-row group",
        "border border-transparent hover:border-border/60",
        draggable && "cursor-grab active:cursor-grabbing"
      )}
      onClick={() => onSelect?.(clip)}
      draggable={draggable}
      onDragStart={(e) => {
        e.dataTransfer.setData("application/x-clipvault-clip", clip.id);
        e.dataTransfer.effectAllowed = "copyMove";
      }}
    >
      <div className="grid h-10 w-10 shrink-0 place-items-center overflow-hidden rounded-md bg-bg-overlay">
        {thumb ? (
          <img src={thumb} alt="" className="h-full w-full object-cover" />
        ) : (
          <Icon className="h-4 w-4 text-fg-muted" />
        )}
      </div>
      <div className="flex min-w-0 flex-1 flex-col">
        <div className="flex items-center gap-2">
          <span className="truncate text-sm text-fg">
            {clip.text_preview || `[${clip.type}]`}
          </span>
          {clip.is_pinned && <Pin className="h-3.5 w-3.5 text-accent" />}
          {clip.is_favorite && <Star className="h-3.5 w-3.5 fill-warning text-warning" />}
        </div>
        <div className="flex items-center gap-2 text-[11px] text-fg-muted">
          {clip.source_app && <span>{clip.source_app}</span>}
          {showCollection && clip.collection_name && (
            <>
              <span>·</span>
              <span>{clip.collection_name}</span>
            </>
          )}
          <span>·</span>
          <span>{formatTimeAgo(clip.created_at)}</span>
          {clip.usage_count > 1 && (
            <>
              <span>·</span>
              <span>used {clip.usage_count}×</span>
            </>
          )}
          {clip.byte_size > 0 && clip.type !== "image" && (
            <>
              <span>·</span>
              <span>{formatBytes(clip.byte_size)}</span>
            </>
          )}
          {clip.tags.length > 0 && (
            <div className="ml-1 flex gap-1">
              {clip.tags.slice(0, 3).map((t) => (
                <span key={t} className="tag-chip">
                  {t}
                </span>
              ))}
            </div>
          )}
        </div>
      </div>
      <div className="flex shrink-0 items-center gap-1 opacity-0 transition-opacity group-hover:opacity-100">
        <button
          title="Copy"
          className="btn-ghost"
          onClick={(e) => {
            e.stopPropagation();
            api.copyClipToClipboard(clip.id);
          }}
        >
          <CopyIcon className="h-3.5 w-3.5" />
        </button>
        <button
          title="Favorite"
          className="btn-ghost"
          onClick={(e) => {
            e.stopPropagation();
            api.toggleFavorite(clip.id, !clip.is_favorite).then(() => {
              qc.invalidateQueries({ queryKey: ["clips"] });
            });
          }}
        >
          <Heart
            className={cn(
              "h-3.5 w-3.5",
              clip.is_favorite && "fill-warning text-warning"
            )}
          />
        </button>
        <button
          title="Delete"
          className="btn-ghost text-danger"
          onClick={(e) => {
            e.stopPropagation();
            if (confirm("Delete this clip?")) {
              api.deleteClip(clip.id).then(() => {
                qc.invalidateQueries({ queryKey: ["clips"] });
                qc.invalidateQueries({ queryKey: ["collections"] });
              });
            }
          }}
        >
          <Trash2 className="h-3.5 w-3.5" />
        </button>
        <button className="btn-ghost" title="More" onClick={(e) => e.stopPropagation()}>
          <MoreHorizontal className="h-3.5 w-3.5" />
        </button>
      </div>
    </div>
  );
}
