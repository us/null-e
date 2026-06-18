import { create } from 'zustand';
import {
  commands,
  type ScanProgressDto,
  type ScanResultDto,
} from '@/lib/tauri';
import { useSystemStore } from './system-store';

const CACHE_KEY = 'null-e:scan-result';

function loadCachedResult(): ScanResultDto | null {
  try {
    const raw = localStorage.getItem(CACHE_KEY);
    return raw ? (JSON.parse(raw) as ScanResultDto) : null;
  } catch {
    return null;
  }
}

function saveCachedResult(result: ScanResultDto | null) {
  try {
    if (result) {
      localStorage.setItem(CACHE_KEY, JSON.stringify(result));
    } else {
      localStorage.removeItem(CACHE_KEY);
    }
  } catch {
    // Ignore storage failures
  }
}

interface ScanState {
  isScanning: boolean;
  progress: ScanProgressDto | null;
  result: ScanResultDto | null;
  error: string | null;

  setProgress: (progress: ScanProgressDto) => void;
  setResult: (result: ScanResultDto) => void;
  setError: (error: string) => void;
  startScan: (roots: string[]) => Promise<void>;
  cancelScan: () => void;
  reset: () => void;
  hasCachedResult: () => boolean;
}

export const useScanStore = create<ScanState>((set) => ({
  isScanning: false,
  progress: null,
  result: loadCachedResult(),
  error: null,

  setProgress: (progress) => set({ progress }),

  setResult: (result) => {
    saveCachedResult(result);
    set({ result, isScanning: false, progress: null });
  },

  setError: (error) =>
    set({ error, isScanning: false, progress: null }),

  startScan: async (roots) => {
    // Keep existing result visible during background rescan
    set((state) => ({
      isScanning: true,
      progress: null,
      error: null,
      // Preserve result so UI stays on results view
      result: state.result,
    }));
    // Kick off system (cleaners + caches) detection CONCURRENTLY with the project scan — they are
    // independent backend commands, so there's no reason to make the user wait for the project
    // walk to finish before the system walk even starts.
    useSystemStore.getState().detectSystem().catch(() => {});
    try {
      await commands.startScan({ roots });
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : String(err),
        isScanning: false,
      });
    }
  },

  cancelScan: () => {
    commands.cancelScan().catch(console.error);
    set({ isScanning: false, progress: null });
  },

  reset: () => {
    saveCachedResult(null);
    set({ isScanning: false, progress: null, result: null, error: null });
  },

  hasCachedResult: () => loadCachedResult() !== null,
}));
