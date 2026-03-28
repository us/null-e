import { create } from 'zustand';
import {
  commands,
  type CleanProgressDto,
  type CleanSummaryDto,
  type CleanConfigDto,
} from '@/lib/tauri';

interface CleanState {
  isCleaning: boolean;
  progress: CleanProgressDto | null;
  summary: CleanSummaryDto | null;
  error: string | null;

  setProgress: (progress: CleanProgressDto) => void;
  setSummary: (summary: CleanSummaryDto) => void;
  setError: (error: string) => void;
  startClean: (targets: string[], config: CleanConfigDto) => Promise<void>;
  cancelClean: () => void;
  reset: () => void;
}

export const useCleanStore = create<CleanState>((set) => ({
  isCleaning: false,
  progress: null,
  summary: null,
  error: null,

  setProgress: (progress) => set({ progress }),

  setSummary: (summary) =>
    set({ summary, isCleaning: false, progress: null }),

  setError: (error) =>
    set({ error, isCleaning: false, progress: null }),

  startClean: async (targets, config) => {
    set({ isCleaning: true, progress: null, summary: null, error: null });
    try {
      await commands.startClean(targets, config);
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : String(err),
        isCleaning: false,
      });
    }
  },

  cancelClean: () => {
    commands.cancelClean().catch(console.error);
    set({ isCleaning: false, progress: null });
  },

  reset: () =>
    set({ isCleaning: false, progress: null, summary: null, error: null }),
}));
