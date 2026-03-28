import { create } from 'zustand';
import {
  commands,
  type ScanProgressDto,
  type ScanResultDto,
} from '@/lib/tauri';

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
}

export const useScanStore = create<ScanState>((set) => ({
  isScanning: false,
  progress: null,
  result: null,
  error: null,

  setProgress: (progress) => set({ progress }),

  setResult: (result) =>
    set({ result, isScanning: false, progress: null }),

  setError: (error) =>
    set({ error, isScanning: false, progress: null }),

  startScan: async (roots) => {
    set({ isScanning: true, progress: null, result: null, error: null });
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

  reset: () =>
    set({ isScanning: false, progress: null, result: null, error: null }),
}));
