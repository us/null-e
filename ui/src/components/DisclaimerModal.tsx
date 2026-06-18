import { useState } from 'react';
import { motion } from 'framer-motion';
import { Shield, AlertTriangle, Info } from 'lucide-react';
import { useUiStore } from '@/stores/ui-store';
import { useFocusTrap } from '@/hooks/useFocusTrap';

const INFO_BLOCKS = [
  {
    icon: Shield,
    title: 'This tool deletes files',
    description:
      'null-e identifies build artifacts, caches, and dependencies and removes them from your disk. Trash is the default method, but deletion is inherently risky.',
  },
  {
    icon: AlertTriangle,
    title: 'No warranty — use at your own risk',
    description:
      'This software is provided as-is with no warranty of any kind. The authors are not responsible for any data loss. Always maintain backups of important work.',
  },
  {
    icon: Info,
    title: 'Trash is not guaranteed',
    description:
      'Moving files to trash works on most systems, but can fail on network drives, certain filesystems, or when disk space is critically low.',
  },
] as const;

export function DisclaimerModal() {
  const [accepted, setAccepted] = useState(false);
  const acceptDisclaimer = useUiStore((s) => s.acceptDisclaimer);
  const trapRef = useFocusTrap<HTMLFormElement>(true);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (accepted) acceptDisclaimer();
  };

  return (
    <>
      {/* Backdrop */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        className="fixed inset-0 bg-black/50 z-40"
      />
      {/* Dialog */}
      <motion.div
        initial={{ opacity: 0, scale: 0.95, y: 10 }}
        animate={{ opacity: 1, scale: 1, y: 0 }}
        transition={{ duration: 0.2 }}
        className="fixed inset-0 flex items-center justify-center z-50 p-4"
      >
        <form
          ref={trapRef}
          onSubmit={handleSubmit}
          tabIndex={-1}
          className="w-full max-w-lg glass-modal outline-none"
          role="dialog"
          aria-modal="true"
          aria-labelledby="disclaimer-title"
          aria-describedby="disclaimer-body"
        >
          {/* Header */}
          <div className="px-6 pt-6 pb-4">
            <h3
              id="disclaimer-title"
              className="text-lg font-semibold text-[var(--color-text)]"
            >
              Welcome to null-e
            </h3>
          </div>

          {/* Body */}
          <div id="disclaimer-body" className="px-6 pb-4 space-y-4">
            {INFO_BLOCKS.map(({ icon: Icon, title, description }) => (
              <div key={title} className="flex gap-3">
                <div className="tint-lav flex items-center justify-center w-10 h-10 shrink-0 rounded-xl">
                  <Icon size={20} className="text-[var(--color-primary)]" />
                </div>
                <div>
                  <p className="text-sm font-medium text-[var(--color-text)]">
                    {title}
                  </p>
                  <p className="text-sm text-[var(--color-text-secondary)]">
                    {description}
                  </p>
                </div>
              </div>
            ))}

            {/* Checkbox */}
            <label className="flex items-center gap-3 cursor-pointer select-none pt-2">
              <input
                type="checkbox"
                checked={accepted}
                onChange={(e) => setAccepted(e.target.checked)}
                className="w-4 h-4 rounded border-[var(--color-border)] accent-[var(--color-primary)]"
              />
              <span className="text-sm text-[var(--color-text)]">
                I understand that null-e deletes files and I accept the risks
              </span>
            </label>
          </div>

          {/* Footer */}
          <div className="flex items-center justify-end px-6 pb-6">
            <button
              type="submit"
              disabled={!accepted}
              className={`pill px-4 py-2 text-sm font-medium text-white bg-[var(--color-primary)] transition-colors focus-visible:ring-2 focus-visible:ring-white outline-none ${
                accepted
                  ? 'hover:bg-[var(--color-primary-hover)]'
                  : 'opacity-50 cursor-not-allowed'
              }`}
            >
              Continue
            </button>
          </div>
        </form>
      </motion.div>
    </>
  );
}
