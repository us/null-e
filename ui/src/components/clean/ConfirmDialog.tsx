import { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { AlertTriangle, Trash2, Archive, X } from 'lucide-react';
import { formatSize } from '@/lib/format';
import { useFocusTrap } from '@/hooks/useFocusTrap';

const SYSTEM_PREFIX = 'system:';

interface ConfirmDialogProps {
  open: boolean;
  onClose: () => void;
  onConfirm: (useTrash: boolean) => void;
  selectedPaths: string[];
  totalSize: number;
}

function stripPrefix(path: string): string {
  return path.startsWith(SYSTEM_PREFIX) ? path.slice(SYSTEM_PREFIX.length) : path;
}

// The word the user must type to arm a permanent delete. A meaningful token reads as intent far
// better than typing back an item count.
const CONFIRM_WORD = 'DELETE';

export function ConfirmDialog({
  open,
  onClose,
  onConfirm,
  selectedPaths,
  totalSize,
}: ConfirmDialogProps) {
  const [useTrash, setUseTrash] = useState(true);
  const [confirmText, setConfirmText] = useState('');
  const trapRef = useFocusTrap<HTMLDivElement>(open);

  // Split the selection so the user can see system/cache items are included — these are easy to
  // sweep in via Select-all or presets without individually inspecting them.
  const systemCount = selectedPaths.filter((p) => p.startsWith(SYSTEM_PREFIX)).length;
  const projectCount = selectedPaths.length - systemCount;
  const confirmArmed = confirmText.trim().toUpperCase() === CONFIRM_WORD;

  // Reset state when dialog closes
  useEffect(() => {
    if (open) {
      setUseTrash(true);
      setConfirmText('');
    }
  }, [open]);

  // Escape key to close
  useEffect(() => {
    if (!open) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    document.addEventListener('keydown', handler);
    return () => document.removeEventListener('keydown', handler);
  }, [open, onClose]);

  return (
    <AnimatePresence>
      {open && (
        <>
          {/* Backdrop */}
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 bg-black/50 z-40"
            onClick={onClose}
          />
          {/* Dialog */}
          <motion.div
            initial={{ opacity: 0, scale: 0.95, y: 10 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.95, y: 10 }}
            transition={{ duration: 0.2 }}
            className="fixed inset-0 flex items-center justify-center z-50 p-4"
          >
            <div
              ref={trapRef}
              tabIndex={-1}
              className="w-full max-w-md glass-modal outline-none"
              role="dialog"
              aria-modal="true"
              aria-labelledby="confirm-dialog-title"
            >
              {/* Header */}
              <div className="flex items-center justify-between px-6 pt-6 pb-4">
                <div className="flex items-center gap-3">
                  <div className="flex items-center justify-center w-10 h-10 rounded-full bg-[var(--color-danger)]/10">
                    <AlertTriangle size={20} className="text-[var(--color-danger)]" />
                  </div>
                  <h3 id="confirm-dialog-title" className="text-lg font-semibold text-[var(--color-text)]">
                    Confirm Clean
                  </h3>
                </div>
                <button
                  onClick={onClose}
                  className="p-1.5 rounded-xl hover:bg-[var(--color-surface-hover)] text-[var(--color-text-muted)] focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] outline-none"
                  aria-label="Close dialog"
                >
                  <X size={18} />
                </button>
              </div>

              {/* Body */}
              <div className="px-6 pb-4 space-y-4">
                <p className="text-sm text-[var(--color-text-secondary)]">
                  You are about to clean{' '}
                  <span className="font-medium text-[var(--color-text)]">
                    {selectedPaths.length} item{selectedPaths.length !== 1 ? 's' : ''}
                  </span>{' '}
                  totaling{' '}
                  <span className="font-medium text-[var(--color-text)]">
                    {formatSize(totalSize)}
                  </span>
                  .
                </p>

                {/* Composition breakdown so the user can tell what kind of items are in the set. */}
                {systemCount > 0 && (
                  <p className="text-xs text-[var(--color-text-muted)]">
                    {projectCount > 0 && (
                      <>{projectCount} project artifact{projectCount !== 1 ? 's' : ''} · </>
                    )}
                    <span className="text-[var(--color-warning-text)] font-medium">
                      {systemCount} system / cache item{systemCount !== 1 ? 's' : ''}
                    </span>{' '}
                    — system caches are regenerated by their apps as needed.
                  </p>
                )}

                {/* Items preview */}
                {selectedPaths.length > 0 && (
                  <div className="max-h-32 overflow-y-auto rounded-xl bg-[var(--color-bg)] p-2 space-y-1">
                    {selectedPaths.slice(0, 10).map((p) => (
                      <p key={p} className="text-xs text-[var(--color-text-muted)] truncate" title={stripPrefix(p)}>
                        {stripPrefix(p)}
                      </p>
                    ))}
                    {selectedPaths.length > 10 && (
                      <p className="text-xs text-[var(--color-text-muted)]">
                        ...and {selectedPaths.length - 10} more
                      </p>
                    )}
                  </div>
                )}

                {/* Trash toggle */}
                <div className="flex items-center justify-between p-3 rounded-xl bg-[var(--color-bg)]">
                  <div className="flex items-center gap-2 text-sm text-[var(--color-text)]">
                    {useTrash ? <Archive size={16} /> : <Trash2 size={16} />}
                    <span>{useTrash ? 'Move to Trash' : 'Delete Permanently'}</span>
                  </div>
                  <button
                    onClick={() => {
                      const next = !useTrash;
                      setUseTrash(next);
                      if (next) setConfirmText('');
                    }}
                    role="switch"
                    aria-checked={useTrash}
                    aria-label="Use trash instead of permanent delete"
                    className={`relative w-10 h-5 rounded-full transition-colors focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] outline-none ${
                      useTrash ? 'bg-[var(--color-safe)]' : 'bg-[var(--color-danger)]'
                    }`}
                  >
                    <span
                      className={`absolute top-0.5 w-4 h-4 rounded-full bg-white transition-transform ${
                        useTrash ? 'left-5' : 'left-0.5'
                      }`}
                    />
                  </button>
                </div>

                {!useTrash && (
                  <div className="space-y-2">
                    <p className="text-xs text-[var(--color-danger)] flex items-center gap-1">
                      <AlertTriangle size={12} />
                      Files will be permanently deleted and cannot be recovered.
                    </p>
                    <p className="text-xs text-[var(--color-danger)]">
                      Make sure you have backups. This action is irreversible.
                    </p>
                    <div className="space-y-1">
                      <label htmlFor="confirm-delete-input" className="text-xs text-[var(--color-text-secondary)]">
                        Type <span className="font-semibold text-[var(--color-text)]">{CONFIRM_WORD}</span> to confirm permanent deletion
                      </label>
                      <input
                        id="confirm-delete-input"
                        type="text"
                        value={confirmText}
                        onChange={(e) => setConfirmText(e.target.value)}
                        placeholder={CONFIRM_WORD}
                        autoComplete="off"
                        autoCapitalize="characters"
                        className="w-full px-3 py-1.5 border border-[var(--color-border)] rounded-xl bg-[var(--color-bg)] text-xs text-[var(--color-text)] outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-primary)]"
                      />
                    </div>
                  </div>
                )}
              </div>

              {/* Footer */}
              <div className="flex items-center justify-end gap-3 px-6 pb-6">
                <button
                  onClick={onClose}
                  className="pill bg-transparent px-4 py-2 text-sm font-medium text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)] transition-colors focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] outline-none"
                >
                  Cancel
                </button>
                <button
                  onClick={() => onConfirm(useTrash)}
                  disabled={!useTrash ? !confirmArmed : false}
                  className="pill px-4 py-2 text-sm font-medium text-white bg-[var(--color-danger)] hover:bg-[var(--color-danger-hover)] transition-colors focus-visible:ring-2 focus-visible:ring-white outline-none disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  Clean {formatSize(totalSize)}
                </button>
              </div>
            </div>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  );
}
