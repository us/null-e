import { useState, useEffect, useCallback, useRef } from 'react';
import { check, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

interface UpdateState {
  available: boolean;
  version: string;
  notes: string;
  downloading: boolean;
  progress: number;
  dismissed: boolean;
}

export function useUpdateCheck() {
  const [state, setState] = useState<UpdateState>({
    available: false,
    version: '',
    notes: '',
    downloading: false,
    progress: 0,
    dismissed: false,
  });
  const updateRef = useRef<Update | null>(null);

  // Check for updates on mount
  useEffect(() => {
    let cancelled = false;

    const doCheck = async () => {
      try {
        const update = await check();
        if (cancelled || !update) return;

        updateRef.current = update;
        setState((prev) => ({
          ...prev,
          available: true,
          version: update.version,
          notes: update.body ?? '',
        }));
      } catch {
        // Silently ignore — offline, rate-limited, or not in Tauri context
      }
    };

    // Delay check by 3 seconds to not block app startup
    const timer = setTimeout(doCheck, 3000);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, []);

  const downloadAndInstall = useCallback(async () => {
    const update = updateRef.current;
    if (!update) {
      setState((prev) => ({ ...prev, downloading: false }));
      return;
    }

    setState((prev) => ({ ...prev, downloading: true, progress: 0 }));

    try {
      let totalBytes = 0;
      let downloadedBytes = 0;

      await update.downloadAndInstall((event) => {
        if (event.event === 'Started' && event.data.contentLength) {
          totalBytes = event.data.contentLength;
        } else if (event.event === 'Progress') {
          downloadedBytes += event.data.chunkLength;
          const pct = totalBytes > 0 ? Math.round((downloadedBytes / totalBytes) * 100) : 0;
          setState((prev) => ({ ...prev, progress: pct }));
        } else if (event.event === 'Finished') {
          setState((prev) => ({ ...prev, progress: 100 }));
        }
      });

      await relaunch();
    } catch (err) {
      console.error('Update failed:', err);
      setState((prev) => ({ ...prev, downloading: false }));
    }
  }, []);

  const dismiss = useCallback(() => {
    setState((prev) => ({ ...prev, dismissed: true }));
  }, []);

  return {
    ...state,
    downloadAndInstall,
    dismiss,
  };
}
