import { useEffect } from 'react';
import { events } from '@/lib/tauri';
import { useScanStore } from '@/stores/scan-store';
import { useSystemStore } from '@/stores/system-store';

/**
 * Hook that listens to Tauri scan events and updates the scan store.
 * Also triggers system detection (cleaners + caches) after scan completes.
 * Should be mounted once at the app level.
 */
export function useScanProgress() {
  const { setProgress, setResult, setError } = useScanStore();

  useEffect(() => {
    let cancelled = false;
    const unlisteners: Array<() => void> = [];

    const setup = async () => {
      const unProgress = await events.onScanProgress((payload) => {
        setProgress(payload);
      });
      if (cancelled) { unProgress(); return; }
      unlisteners.push(unProgress);

      const unComplete = await events.onScanComplete((payload) => {
        setResult(payload);
        // Detect system-level items (cleaners + caches) in background
        useSystemStore.getState().detectSystem().catch(console.error);
      });
      if (cancelled) { unComplete(); return; }
      unlisteners.push(unComplete);

      const unError = await events.onScanError((payload) => {
        setError(payload);
      });
      if (cancelled) { unError(); return; }
      unlisteners.push(unError);
    };

    setup().catch(console.error);

    return () => {
      cancelled = true;
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, [setProgress, setResult, setError]);
}
