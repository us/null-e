import { useEffect } from 'react';
import { events } from '@/lib/tauri';
import { useScanStore } from '@/stores/scan-store';

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
        // System detection (cleaners + caches) is now kicked off at scan START (see scan-store),
        // so it runs CONCURRENTLY with the project scan instead of waiting for it to finish.
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
