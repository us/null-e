import { Sun, Moon, Monitor, Settings, RotateCcw } from 'lucide-react';
import { useUiStore } from '@/stores/ui-store';
import { useScanStore } from '@/stores/scan-store';

export function AppBar() {
  const { appState, theme, toggleTheme, setSettingsOpen } = useUiStore();
  const { isScanning } = useScanStore();
  const showRescan = appState === 'results' || appState === 'done';

  const handleRescan = () => {
    useScanStore.getState().reset();
    useUiStore.getState().setAppState('welcome');
  };

  const themeIcon = theme === 'dark' ? <Sun size={16} /> : theme === 'light' ? <Moon size={16} /> : <Monitor size={16} />;
  const themeLabel = `Current theme: ${theme}`;

  return (
    <header
      data-tauri-drag-region
      className="flex items-center justify-between px-5 h-12 shrink-0 border-b border-[var(--color-border)] glass"
    >
      <div data-tauri-drag-region className="flex items-center gap-2">
        <img src="/logo.png" alt="null-e" width={22} height={22} className="rounded" />
        <span className="text-base font-semibold text-[var(--color-text)]">
          null-e
        </span>
      </div>

      <div className="flex items-center gap-1">
        {showRescan && !isScanning && (
          <button
            onClick={handleRescan}
            className="p-2.5 rounded-lg hover:bg-[var(--color-surface-hover)] text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] outline-none"
            aria-label="New scan"
            title="New scan"
          >
            <RotateCcw size={16} />
          </button>
        )}
        <button
          onClick={toggleTheme}
          className="p-2.5 rounded-lg hover:bg-[var(--color-surface-hover)] text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] outline-none"
          aria-label={themeLabel}
          title="Toggle theme"
        >
          {themeIcon}
        </button>
        <button
          onClick={() => setSettingsOpen(true)}
          className="p-2.5 rounded-lg hover:bg-[var(--color-surface-hover)] text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] outline-none"
          aria-label="Settings"
          title="Settings"
        >
          <Settings size={16} />
        </button>
      </div>
    </header>
  );
}
