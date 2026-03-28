import { useEffect, useState } from 'react';
import { Search, Settings, AlertCircle } from 'lucide-react';
import { motion } from 'framer-motion';
import { commands, type DiskInfoDto } from '@/lib/tauri';
import { formatSize } from '@/lib/format';
import { useScanStore } from '@/stores/scan-store';
import { useUiStore } from '@/stores/ui-store';

export function WelcomeView() {
  const [diskInfo, setDiskInfo] = useState<DiskInfoDto | null>(null);
  const [scanPaths, setScanPaths] = useState<string[]>([]);
  const { startScan, error: scanError } = useScanStore();

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
        <span className="text-5xl font-bold text-[var(--color-text)] tracking-tight">
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

      {/* Error message */}
      {scanError && (
        <motion.div
          initial={{ opacity: 0, y: -10 }}
          animate={{ opacity: 1, y: 0 }}
          className="flex items-center gap-2 px-4 py-3 rounded-xl bg-red-500/10 text-[var(--color-danger)] text-sm max-w-md"
        >
          <AlertCircle size={16} className="shrink-0" />
          <span>{scanError}</span>
        </motion.div>
      )}

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
          : 'No scan paths configured — add paths in settings'}
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
