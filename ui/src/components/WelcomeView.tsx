import { useEffect, useState } from 'react';
import { Search, Settings, AlertCircle, ShieldAlert, ExternalLink, RotateCcw } from 'lucide-react';
import { motion } from 'framer-motion';
import { relaunch } from '@tauri-apps/plugin-process';
import { commands, type DiskInfoDto } from '@/lib/tauri';
import { formatSize } from '@/lib/format';
import { useScanStore } from '@/stores/scan-store';
import { useUiStore } from '@/stores/ui-store';

export function WelcomeView() {
  const [diskInfo, setDiskInfo] = useState<DiskInfoDto | null>(null);
  const [scanPaths, setScanPaths] = useState<string[]>([]);
  const { startScan, error: scanError } = useScanStore();
  const fdaStatus = useUiStore((s) => s.fdaStatus);
  const fdaDismissed = useUiStore((s) => s.fdaDismissed);
  const fdaLostAfterUpdate = useUiStore((s) => s.fdaLostAfterUpdate);

  useEffect(() => {
    commands.getDiskInfo().then(setDiskInfo).catch(console.error);
    commands
      .getConfig()
      .then((raw) => {
        // Config structure: { general: { default_paths: [...] }, scan: { ... }, ... }
        const general = raw.general as Record<string, unknown> | undefined;
        const paths = (general?.default_paths as string[]) ?? [];
        setScanPaths(paths);
      })
      .catch(console.error);
  }, []);

  const handleScan = async () => {
    // Clear previous errors
    useScanStore.getState().reset();
    useUiStore.getState().setAppState('scanning');

    // Pass configured paths (backend falls back to defaults if empty)
    await startScan(scanPaths);

    // If startScan set an error (Tauri command rejected), go back to welcome
    const { error } = useScanStore.getState();
    if (error) {
      useUiStore.getState().setAppState('welcome');
    }
  };

  const freePercent = diskInfo
    ? ((diskInfo.available / diskInfo.total) * 100).toFixed(1)
    : null;

  const openFdaSettings = async () => {
    try {
      await commands.openPrivacySettings();
    } catch (err) {
      console.error('Failed to open Full Disk Access settings:', err);
    }
  };

  const handleRelaunch = async () => {
    try {
      await relaunch();
    } catch (err) {
      console.error('Failed to relaunch:', err);
    }
  };

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.4 }}
      className="flex flex-col items-center justify-center h-full gap-8 px-6"
    >
      {/* Logo */}
      <div className="flex flex-col items-center gap-3">
        <img src="/logo.png" alt="null-e mascot" width={240} height={240} className="rounded-3xl" />
        <span className="display text-5xl text-[var(--color-text)] tracking-tight">
          null-e
        </span>
      </div>

      {/* Disk info */}
      {diskInfo && (
        <div className="flex flex-col items-center gap-2">
          <div className="w-64 h-2 rounded-full bg-[var(--color-bg-tertiary)] overflow-hidden">
            <div
              className="h-full rounded-full bg-gradient-to-r from-[var(--color-primary)] to-[var(--color-safe)] transition-all"
              style={{
                width: `${100 - Number(freePercent)}%`,
              }}
            />
          </div>
          <p className="text-sm text-[var(--color-text-secondary)]">
            Your disk has{' '}
            <span className="font-semibold text-[var(--color-text)]">
              {formatSize(diskInfo.available)}
            </span>{' '}
            free of {formatSize(diskInfo.total)} ({freePercent}% available)
          </p>
        </div>
      )}

      {fdaStatus === 'not_granted' && !fdaDismissed && (
        <motion.div
          initial={{ opacity: 0, y: 8 }}
          animate={{ opacity: 1, y: 0 }}
          className="w-full max-w-2xl rounded-2xl border border-[var(--color-warning-border)] bg-[var(--color-warning-surface)] px-5 py-4"
        >
          <div className="flex gap-4">
            <div className="mt-0.5 rounded-xl bg-[var(--color-warning-surface)] p-2 text-[var(--color-warning-text)]">
              <ShieldAlert size={18} />
            </div>
            <div className="flex-1">
              <div className="flex flex-col gap-1">
                <h3 className="text-sm font-semibold text-[var(--color-text)]">
                  {fdaLostAfterUpdate
                    ? 'An update reset Full Disk Access'
                    : 'Enable Full Disk Access'}
                </h3>
                <p className="text-sm text-[var(--color-text-secondary)]">
                  {fdaLostAfterUpdate
                    ? 'You granted Full Disk Access before, but updating an unsigned app makes macOS reset the permission. Re-enable null-e and relaunch.'
                    : 'macOS hides system and other-app caches behind Full Disk Access. Without it, those deletions fail with “Operation not permitted”.'}
                </p>
              </div>
              <ol className="mt-3 space-y-1.5 text-sm text-[var(--color-text-secondary)]">
                <li><span className="font-semibold text-[var(--color-text)]">1.</span> Click <span className="font-medium">Open Settings</span> below.</li>
                <li><span className="font-semibold text-[var(--color-text)]">2.</span> Enable <span className="font-medium">null-e</span> in the Full Disk Access list (use <span className="font-medium">+</span> to add it if it isn’t there).</li>
                <li><span className="font-semibold text-[var(--color-text)]">3.</span> Click <span className="font-medium">Relaunch</span> — macOS only applies the grant to a freshly launched app.</li>
              </ol>
              <div className="mt-4 flex flex-wrap gap-2">
                <button
                  onClick={() => { void openFdaSettings(); }}
                  className="inline-flex items-center gap-2 rounded-xl bg-[var(--color-warning)] px-3.5 py-2 text-sm font-medium text-[var(--color-warning-strong-text)] transition-colors hover:bg-[var(--color-warning-hover)]"
                >
                  <ExternalLink size={14} />
                  Open Settings
                </button>
                <button
                  onClick={() => { void handleRelaunch(); }}
                  className="inline-flex items-center gap-2 rounded-xl border border-[var(--color-border)] px-3.5 py-2 text-sm font-medium text-[var(--color-text)] transition-colors hover:bg-[var(--color-surface-hover)]"
                >
                  <RotateCcw size={14} />
                  Relaunch
                </button>
                <button
                  onClick={() => useUiStore.getState().dismissFda()}
                  className="rounded-xl border border-[var(--color-border)] px-3.5 py-2 text-sm text-[var(--color-text-secondary)] transition-colors hover:bg-[var(--color-surface-hover)]"
                >
                  Not now
                </button>
              </div>
            </div>
          </div>
        </motion.div>
      )}

      {/* Error message — categorized with a next step instead of a raw backend string */}
      {scanError && (() => {
        const lower = scanError.toLowerCase();
        const isPermission =
          lower.includes('permission') ||
          lower.includes('operation not permitted') ||
          lower.includes('not permitted') ||
          lower.includes('os error 1') ||
          lower.includes('os error 13') ||
          lower.includes('denied');
        return (
          <motion.div
            initial={{ opacity: 0, y: -10 }}
            animate={{ opacity: 1, y: 0 }}
            className="w-full max-w-md rounded-xl border border-[var(--color-danger)]/25 bg-[var(--color-danger)]/10 px-4 py-3"
          >
            <div className="flex items-start gap-2">
              <AlertCircle size={16} className="mt-0.5 shrink-0 text-[var(--color-danger)]" />
              <div className="flex-1">
                <p className="text-sm font-medium text-[var(--color-text)]">
                  {isPermission ? 'Scan blocked by macOS permissions' : 'Scan failed'}
                </p>
                <p className="mt-1 text-xs text-[var(--color-text-secondary)]">
                  {isPermission
                    ? 'macOS denied access to some locations. Grant Full Disk Access to null-e, relaunch, then scan again.'
                    : scanError}
                </p>
                <div className="mt-3 flex flex-wrap gap-2">
                  {isPermission && (
                    <button
                      onClick={() => { void openFdaSettings(); }}
                      className="inline-flex items-center gap-1.5 rounded-lg bg-[var(--color-warning)] px-3 py-1.5 text-xs font-medium text-[var(--color-warning-strong-text)] transition-colors hover:bg-[var(--color-warning-hover)]"
                    >
                      <ExternalLink size={13} />
                      Open Full Disk Access
                    </button>
                  )}
                  <button
                    onClick={() => { void handleScan(); }}
                    className="inline-flex items-center gap-1.5 rounded-lg border border-[var(--color-border)] px-3 py-1.5 text-xs font-medium text-[var(--color-text)] transition-colors hover:bg-[var(--color-surface-hover)]"
                  >
                    <RotateCcw size={13} />
                    Retry
                  </button>
                </div>
              </div>
            </div>
          </motion.div>
        );
      })()}

      {/* Scan CTA */}
      <button
        onClick={handleScan}
        className="flex items-center gap-3 px-10 py-4 rounded-2xl text-white font-semibold text-base bg-[var(--color-primary)] hover:bg-[var(--color-primary-hover)] transition-all shadow-lg hover:shadow-xl hover:scale-[1.02] active:scale-[0.98]"
      >
        <Search size={20} />
        Scan for waste
      </button>

      {/* Current scan paths info */}
      <p className="text-xs text-[var(--color-text-muted)] max-w-sm text-center">
        {scanPaths.length > 0
          ? `Scanning: ${scanPaths.join(', ')}`
          : 'Will scan your home folder — add specific paths in settings to narrow it down'}
      </p>

      {/* Settings link */}
      <button
        onClick={() => useUiStore.getState().setSettingsOpen(true)}
        className="flex items-center gap-1.5 text-xs text-[var(--color-text-muted)] hover:text-[var(--color-text-secondary)] transition-colors"
      >
        <Settings size={12} />
        Change scan paths
      </button>
    </motion.div>
  );
}
