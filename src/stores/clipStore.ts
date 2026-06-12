import { create } from "zustand";

interface ClipState {
  selectedId: string | null;
  setSelected: (id: string | null) => void;
  lastSearch: string;
  setLastSearch: (q: string) => void;
}

export const useClipStore = create<ClipState>((set) => ({
  selectedId: null,
  setSelected: (id) => set({ selectedId: id }),
  lastSearch: "",
  setLastSearch: (q) => set({ lastSearch: q }),
}));
