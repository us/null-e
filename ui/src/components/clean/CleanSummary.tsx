import { motion, AnimatePresence } from 'framer-motion';
import { CheckCircle, AlertCircle, X } from 'lucide-react';
import { formatSize } from '@/lib/format';
import type { CleanSummaryDto } from '@/lib/tauri';

interface CleanSummaryProps {
  summary: CleanSummaryDto | null;
  onClose: () => void;
}

export function CleanSummary({ summary, onClose }: CleanSummaryProps) {
  if (!summary) return null;

  const hasFailures = summary.failed > 0;
  const allFailed = summary.succeeded === 0 && summary.failed > 0;

  return (
    <AnimatePresence>
      {summary && (
        <>
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 bg-black/50 z-40"
            onClick={onClose}
          />
          <motion.div
            initial={{ opacity: 0, scale: 0.95, y: 10 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.95, y: 10 }}
            transition={{ duration: 0.2 }}
            className="fixed inset-0 flex items-center justify-center z-50 p-4"
          >
            <div className="w-full max-w-sm rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] shadow-xl">
              {/* Header */}
              <div className="flex items-center justify-between px-6 pt-6 pb-2">
                <div className="flex items-center gap-3">
                  {allFailed ? (
                    <AlertCircle size={24} className="text-[var(--color-danger)]" />
                  ) : hasFailures ? (
                    <AlertCircle size={24} className="text-[var(--color-warning)]" />
                  ) : (
                    <CheckCircle size={24} className="text-[var(--color-safe)]" />
                  )}
                  <h3 className="text-lg font-semibold text-[var(--color-text)]">
                    {allFailed ? 'Clean Failed' : 'Clean Complete'}
                  </h3>
                </div>
                <button
                  onClick={onClose}
                  className="p-1 rounded-lg hover:bg-[var(--color-surface-hover)] text-[var(--color-text-muted)]"
                >
                  <X size={18} />
                </button>
              </div>

              {/* Body */}
              <div className="px-6 py-4 space-y-3">
                <div className="text-center py-2">
                  <p className="text-3xl font-bold bg-gradient-to-r from-[var(--color-primary)] to-[var(--color-safe)] bg-clip-text text-transparent">
                    {formatSize(summary.bytes_freed)}
                  </p>
                  <p className="text-sm text-[var(--color-text-secondary)] mt-1">space freed</p>
                </div>

                <div className="grid grid-cols-3 gap-2 text-center">
                  <div className="p-2 rounded-lg bg-[var(--color-bg)]">
                    <p className="text-lg font-semibold text-[var(--color-text)]">
                      {summary.total_items}
                    </p>
                    <p className="text-xs text-[var(--color-text-muted)]">Total</p>
                  </div>
                  <div className="p-2 rounded-lg bg-[var(--color-bg)]">
                    <p className="text-lg font-semibold text-[var(--color-safe)]">
                      {summary.succeeded}
                    </p>
                    <p className="text-xs text-[var(--color-text-muted)]">Cleaned</p>
                  </div>
                  <div className="p-2 rounded-lg bg-[var(--color-bg)]">
                    <p
                      className={`text-lg font-semibold ${
                        summary.failed > 0
                          ? 'text-[var(--color-danger)]'
                          : 'text-[var(--color-text)]'
                      }`}
                    >
                      {summary.failed}
                    </p>
                    <p className="text-xs text-[var(--color-text-muted)]">Failed</p>
                  </div>
                </div>

                {summary.used_trash && (
                  <p className="text-xs text-[var(--color-text-muted)] text-center">
                    Items were moved to Trash
                  </p>
                )}
              </div>

              {/* Footer */}
              <div className="px-6 pb-6">
                <button
                  onClick={onClose}
                  className="w-full px-4 py-2 rounded-lg text-sm font-medium text-white bg-[var(--color-primary)] hover:bg-[var(--color-primary-hover)] transition-colors"
                >
                  Done
                </button>
              </div>
            </div>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  );
}
