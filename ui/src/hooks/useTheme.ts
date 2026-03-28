import { useEffect } from 'react';
import { useUiStore } from '@/stores/ui-store';

/**
 * Hook for managing dark/light theme on the document root.
 * Should be mounted once at the app level.
 */
export function useTheme() {
  const theme = useUiStore((s) => s.theme);

  useEffect(() => {
    const root = document.documentElement;

    const applyTheme = (isDark: boolean) => {
      if (isDark) {
        root.classList.add('dark');
      } else {
        root.classList.remove('dark');
      }
    };

    if (theme === 'system') {
      const mq = window.matchMedia('(prefers-color-scheme: dark)');
      applyTheme(mq.matches);

      const handler = (e: MediaQueryListEvent) => applyTheme(e.matches);
      mq.addEventListener('change', handler);
      return () => mq.removeEventListener('change', handler);
    }

    applyTheme(theme === 'dark');
  }, [theme]);
}
