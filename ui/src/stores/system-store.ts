import { create } from 'zustand';
import { commands, type CleanableItemDto, type GlobalCacheDto } from '@/lib/tauri';

interface SystemState {
  cleaners: CleanableItemDto[];
  caches: GlobalCacheDto[];
  isDetecting: boolean;
  error: string | null;

  detectSystem: () => Promise<void>;
  reset: () => void;
}

export const useSystemStore = create<SystemState>((set, get) => ({
  cleaners: [],
  caches: [],
  isDetecting: false,
  error: null,

  detectSystem: async () => {
    if (get().isDetecting) return;
    set({ isDetecting: true, error: null });
    try {
      const [cleaners, caches] = await Promise.all([
        commands.detectCleaners(),
        commands.detectCaches(),
      ]);
      set({ cleaners, caches, isDetecting: false });
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : String(err),
        isDetecting: false,
      });
    }
  },

  reset: () => set({ cleaners: [], caches: [], isDetecting: false, error: null }),
}));
