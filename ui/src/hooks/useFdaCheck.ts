import { useEffect } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { commands } from '@/lib/tauri';
import { useUiStore } from '@/stores/ui-store';

export function useFdaCheck() {
  useEffect(() => {
    let cancelled = false;

    const refreshStatus = async () => {
      try {
        const status = await commands.checkFdaStatus();
        if (!cancelled) {
          useUiStore.getState().setFdaStatus(status.status);
        }
      } catch {
        if (!cancelled) {
          useUiStore.getState().setFdaStatus('unknown');
        }
      }
    };

    void refreshStatus();

    const unlistenPromise = getCurrentWindow().onFocusChanged(({ payload: focused }) => {
      if (focused) {
        void refreshStatus();
      }
    });

    return () => {
      cancelled = true;
      void unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);
}
