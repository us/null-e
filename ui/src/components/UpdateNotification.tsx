import { motion, AnimatePresence } from 'framer-motion';
import { Download, X, RefreshCw } from 'lucide-react';
import { useUpdateCheck } from '@/hooks/useUpdateCheck';

export function UpdateNotification() {
  const { available, version, downloading, progress, dismissed, downloadAndInstall, dismiss } =
    useUpdateCheck();

  const show = available && !dismissed;

  return (
    <AnimatePresence>
      {show && (
        <motion.div
          initial={{ height: 0, opacity: 0 }}
          animate={{ height: 'auto', opacity: 1 }}
          exit={{ height: 0, opacity: 0 }}
          transition={{ duration: 0.2 }}
          className="shrink-0 overflow-hidden"
        >
          <div className="flex items-center gap-3 px-5 py-2.5 bg-[var(--color-primary)]/10 border-b border-[var(--color-primary)]/20">
            {downloading ? (
              <RefreshCw size={14} className="animate-spin text-[var(--color-primary)]" />
            ) : (
              <Download size={14} className="text-[var(--color-primary)]" />
            )}

            <span className="text-xs text-[var(--color-text)]">
              {downloading
                ? `Downloading v${version}... ${progress}%`
                : `Update available: v${version}`}
            </span>

            {/* Progress bar during download */}
            {downloading && (
              <div className="flex-1 max-w-[120px] h-1.5 rounded-full bg-[var(--color-bg-tertiary)] overflow-hidden">
                <motion.div
                  className="h-full rounded-full bg-[var(--color-primary)]"
                  initial={{ width: 0 }}
                  animate={{ width: `${progress}%` }}
                  transition={{ duration: 0.3 }}
                />
              </div>
            )}

            <div className="ml-auto flex items-center gap-2">
              {!downloading && (
                <>
                  <button
                    onClick={downloadAndInstall}
                    className="text-xs font-medium px-3 py-1 rounded-md bg-[var(--color-primary)] text-white hover:opacity-90 transition-opacity focus-visible:ring-2 focus-visible:ring-white outline-none"
                  >
                    Update & Restart
                  </button>
                  <button
                    onClick={dismiss}
                    className="p-1 rounded hover:bg-[var(--color-surface-hover)] text-[var(--color-text-muted)] focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] outline-none"
                    aria-label="Dismiss update notification"
                  >
                    <X size={14} />
                  </button>
                </>
              )}
            </div>
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
