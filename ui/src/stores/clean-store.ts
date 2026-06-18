import { create } from 'zustand';
import {
  commands,
  type CleanProgressDto,
  type CleanSummaryDto,
  type CleanConfigDto,
} from '@/lib/tauri';

interface CleanState {
  isCleaning: boolean;
  /** True between hitting Cancel and the backend confirming with a (partial) summary. Drives the
   * "Stopping…" overlay state so cancellation isn't a silent instant vanish. */
  canceling: boolean;
  /** Whether the active/last clean used Trash (vs permanent delete) — lets the overlay say
   * "moved to Trash" instead of the misleading "freed" while Trash mode frees nothing yet. */
  usedTrash: boolean;
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
  canceling: false,
  usedTrash: true,
  progress: null,
  summary: null,
  error: null,

  setProgress: (progress) => set({ progress }),

  setSummary: (summary) =>
    set({ summary, isCleaning: false, canceling: false, progress: null }),

  setError: (error) =>
    set({ error, isCleaning: false, canceling: false, progress: null }),

  startClean: async (targets, config) => {
    set({
      isCleaning: true,
      canceling: false,
      usedTrash: config.use_trash,
      progress: null,
      summary: null,
      error: null,
    });
    try {
      await commands.startClean(targets, config);
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : String(err),
        isCleaning: false,
        canceling: false,
      });
    }
  },

  cancelClean: () => {
    // Ask the backend to stop, but KEEP the overlay up in a "Stopping…" state. The backend always
    // emits a partial clean:complete summary after a cancel, which drives setSummary() → the
    // results/celebration screen showing exactly what was actually cleaned. Tearing the overlay
    // down here would hide that and imply nothing happened.
    commands.cancelClean().catch(console.error);
    set({ canceling: true });
  },

  reset: () =>
    set({
      isCleaning: false,
      canceling: false,
      usedTrash: true,
      progress: null,
      summary: null,
      error: null,
    }),
}));
