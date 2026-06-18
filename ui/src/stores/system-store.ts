import { create } from 'zustand';
import { commands, type CleanableItemDto, type GlobalCacheDto } from '@/lib/tauri';

const CACHE_KEY = 'null-e:system-result';

interface CachedSystem {
  cleaners: CleanableItemDto[];
  caches: GlobalCacheDto[];
}

function loadCached(): CachedSystem | null {
  try {
    const raw = localStorage.getItem(CACHE_KEY);
    return raw ? (JSON.parse(raw) as CachedSystem) : null;
  } catch {
    return null;
  }
}

function saveCache(cleaners: CleanableItemDto[], caches: GlobalCacheDto[]) {
  try {
    localStorage.setItem(CACHE_KEY, JSON.stringify({ cleaners, caches }));
  } catch {
    // Ignore storage failures
  }
}

let currentDetection: Promise<void> | null = null;
let queuedDetection: Promise<void> | null = null;

const cached = loadCached();

interface SystemState {
  cleaners: CleanableItemDto[];
  caches: GlobalCacheDto[];
  /** Cleaners that errored during detection (e.g. permission issues) — surfaced, not hidden. */
  skippedCleaners: string[];
  isDetecting: boolean;
  error: string | null;

  detectSystem: () => Promise<void>;
  reset: () => void;
}

export const useSystemStore = create<SystemState>((set, get) => ({
  cleaners: cached?.cleaners ?? [],
  caches: cached?.caches ?? [],
  skippedCleaners: [],
  isDetecting: false,
  error: null,

  detectSystem: async () => {
    if (currentDetection) {
      queuedDetection ??= currentDetection.finally(() => {
        queuedDetection = null;
      }).then(() => get().detectSystem());
      return queuedDetection;
    }

    currentDetection = (async () => {
      set({ isDetecting: true, error: null });
      try {
        const [cleanersResult, caches] = await Promise.all([
          commands.detectCleaners(),
          commands.detectCaches(),
        ]);
        const cleaners = cleanersResult.items;
        saveCache(cleaners, caches);
        set({
          cleaners,
          caches,
          skippedCleaners: cleanersResult.skipped,
          isDetecting: false,
        });
      } catch (err) {
        set({
          error: err instanceof Error ? err.message : String(err),
          isDetecting: false,
        });
      } finally {
        currentDetection = null;
      }
    })();

    return currentDetection;
  },

  reset: () => {
    try { localStorage.removeItem(CACHE_KEY); } catch { /* ignore */ }
    set({ cleaners: [], caches: [], skippedCleaners: [], isDetecting: false, error: null });
  },
}));
