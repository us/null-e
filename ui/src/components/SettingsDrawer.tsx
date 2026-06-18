import { useEffect, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X, Save, Loader2, CheckCircle, AlertCircle, Download, RefreshCw } from 'lucide-react';
import { commands } from '@/lib/tauri';
import { useUiStore, type Theme } from '@/stores/ui-store';
import { useFocusTrap } from '@/hooks/useFocusTrap';
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

interface AppConfig {
  scan_paths: string[];
  max_depth: number;
  min_size: number;
  use_trash: boolean;
  protection_level: string;
}

const defaultConfig: AppConfig = {
  scan_paths: [],
  max_depth: 10,
  min_size: 0,
  use_trash: true,
  protection_level: 'normal',
};

export function SettingsDrawer() {
  const { settingsOpen, setSettingsOpen, theme, setTheme } = useUiStore();
  const [config, setConfig] = useState<AppConfig>(defaultConfig);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);
  const [newPath, setNewPath] = useState('');
  const [disclaimerOpen, setDisclaimerOpen] = useState(false);
  const [updateStatus, setUpdateStatus] = useState<'idle' | 'checking' | 'available' | 'downloading' | 'none'>('idle');
  const [updateVersion, setUpdateVersion] = useState('');

  const checkForUpdates = async () => {
    setUpdateStatus('checking');
    try {
      const update = await check();
      if (update) {
        setUpdateVersion(update.version);
        setUpdateStatus('available');
      } else {
        setUpdateStatus('none');
        setTimeout(() => setUpdateStatus('idle'), 3000);
      }
    } catch {
      setUpdateStatus('none');
      setTimeout(() => setUpdateStatus('idle'), 3000);
    }
  };

  const installUpdate = async () => {
    setUpdateStatus('downloading');
    try {
      const update = await check();
      if (update) {
        await update.downloadAndInstall();
        await relaunch();
      }
    } catch (err) {
      console.error('Update install failed:', err);
      setUpdateStatus('available');
    }
  };

  useEffect(() => {
    if (!settingsOpen) return;
    setLoading(true);
    commands
      .getConfig()
      .then((raw) => {
        setConfig({
          scan_paths: (raw.scan_paths as string[]) ?? defaultConfig.scan_paths,
          max_depth: (raw.max_depth as number) ?? defaultConfig.max_depth,
          min_size: (raw.min_size as number) ?? defaultConfig.min_size,
          use_trash: (raw.use_trash as boolean) ?? defaultConfig.use_trash,
          protection_level:
            (raw.protection_level as string) ?? defaultConfig.protection_level,
        });
      })
      .catch((err) =>
        setError(err instanceof Error ? err.message : String(err))
      )
      .finally(() => setLoading(false));
  }, [settingsOpen]);

  const saveSettings = async () => {
    setSaving(true);
    setError(null);
    setSaved(false);
    try {
      await commands.saveConfig(config as unknown as Record<string, unknown>);
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSaving(false);
    }
  };

  const addPath = () => {
    const trimmed = newPath.trim();
    if (trimmed && !config.scan_paths.includes(trimmed)) {
      setConfig({ ...config, scan_paths: [...config.scan_paths, trimmed] });
      setNewPath('');
    }
  };

  const removePath = (index: number) => {
    setConfig({
      ...config,
      scan_paths: config.scan_paths.filter((_, i) => i !== index),
    });
  };

  const trapRef = useFocusTrap<HTMLDivElement>(settingsOpen);

  // Escape closes the drawer, matching ConfirmDialog's behavior.
  useEffect(() => {
    if (!settingsOpen) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setSettingsOpen(false);
    };
    document.addEventListener('keydown', handler);
    return () => document.removeEventListener('keydown', handler);
  }, [settingsOpen, setSettingsOpen]);

  return (
    <AnimatePresence>
      {settingsOpen && (
        <>
          {/* Backdrop */}
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 bg-black/40 z-40"
            onClick={() => setSettingsOpen(false)}
          />
          {/* Drawer */}
          <motion.div
            ref={trapRef}
            tabIndex={-1}
            role="dialog"
            aria-modal="true"
            aria-label="Settings"
            initial={{ x: '100%' }}
            animate={{ x: 0 }}
            exit={{ x: '100%' }}
            transition={{ type: 'spring', damping: 30, stiffness: 300 }}
            className="fixed right-0 top-0 bottom-0 w-[380px] max-w-[90vw] z-50 bg-[var(--color-surface-solid)] border-l border-[var(--color-border)] shadow-2xl overflow-y-auto outline-none"
          >
            {/* Header */}
            <div className="flex items-center justify-between px-5 py-4 border-b border-[var(--color-border)]">
              <h2 className="text-base font-semibold text-[var(--color-text)]">
                Settings
              </h2>
              <div className="flex items-center gap-2">
                <button
                  onClick={saveSettings}
                  disabled={saving || loading}
                  className="pill flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-white bg-[var(--color-primary)] hover:bg-[var(--color-primary-hover)] disabled:opacity-50 disabled:cursor-not-allowed transition-colors focus-visible:ring-2 focus-visible:ring-white outline-none"
                >
                  {saving ? (
                    <Loader2 size={12} className="animate-spin" />
                  ) : saved ? (
                    <CheckCircle size={12} />
                  ) : (
                    <Save size={12} />
                  )}
                  {saved ? 'Saved!' : 'Save'}
                </button>
                <button
                  onClick={() => setSettingsOpen(false)}
                  aria-label="Close settings"
                  title="Close settings"
                  className="p-2 rounded-lg hover:bg-[var(--color-surface-hover)] text-[var(--color-text-muted)] focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] outline-none"
                >
                  <X size={16} />
                </button>
              </div>
            </div>

            {loading ? (
              <div className="flex items-center justify-center py-12">
                <Loader2
                  size={20}
                  className="animate-spin text-[var(--color-primary)]"
                />
              </div>
            ) : (
              <div className="p-5 flex flex-col gap-4">
                {error && (
                  <div className="flex items-center gap-2 p-3 rounded-xl bg-[var(--color-danger)]/10 text-[var(--color-danger)] text-xs">
                    <AlertCircle size={14} />
                    <span>{error}</span>
                  </div>
                )}

                {/* Scan Paths */}
                <Section title="Scan Paths">
                  <div className="space-y-2">
                    <div className="flex gap-2">
                      <input
                        type="text"
                        value={newPath}
                        onChange={(e) => setNewPath(e.target.value)}
                        onKeyDown={(e) => e.key === 'Enter' && addPath()}
                        placeholder="/Users/you/code"
                        className="flex-1 px-3 py-1.5 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-solid)] text-xs text-[var(--color-text)] placeholder:text-[var(--color-text-muted)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]"
                      />
                      <button
                        onClick={addPath}
                        className="pill px-3 py-1.5 text-xs text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)] transition-colors focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] outline-none"
                      >
                        Add
                      </button>
                    </div>
                    {config.scan_paths.length > 0 ? (
                      <div className="bento overflow-hidden">
                        {config.scan_paths.map((path, i) => (
                          <div
                            key={path}
                            className="bento-row flex items-center justify-between px-4 py-2.5"
                          >
                            <span className="text-[11px] text-[var(--color-text-secondary)] truncate" title={path}>
                              {path}
                            </span>
                            <button
                              onClick={() => removePath(i)}
                              aria-label={`Remove scan path ${path}`}
                              className="pill px-2 py-1 text-xs text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10 transition-colors shrink-0 ml-2 focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] outline-none"
                            >
                              Remove
                            </button>
                          </div>
                        ))}
                      </div>
                    ) : (
                      <p className="text-[11px] text-[var(--color-text-muted)]">
                        No paths configured — will use system defaults
                      </p>
                    )}
                  </div>
                </Section>

                {/* Scan Options */}
                <Section title="Scan Options">
                  <Field label={`Max Depth: ${config.max_depth}`}>
                    <input
                      type="range"
                      min={1}
                      max={20}
                      value={config.max_depth}
                      onChange={(e) =>
                        setConfig({
                          ...config,
                          max_depth: Number(e.target.value),
                        })
                      }
                      className="w-full accent-[var(--color-primary)]"
                    />
                  </Field>
                  <Field
                    label={`Min Size: ${config.min_size === 0 ? 'No limit' : `${config.min_size} MB`}`}
                  >
                    <input
                      type="range"
                      min={0}
                      max={500}
                      step={10}
                      value={config.min_size}
                      onChange={(e) =>
                        setConfig({
                          ...config,
                          min_size: Number(e.target.value),
                        })
                      }
                      className="w-full accent-[var(--color-primary)]"
                    />
                  </Field>
                </Section>

                {/* Clean Options */}
                <Section title="Clean Options">
                  <div className="flex items-center justify-between">
                    <span className="text-xs text-[var(--color-text-secondary)]">
                      Use Trash
                    </span>
                    <Toggle
                      checked={config.use_trash}
                      onChange={(v) =>
                        setConfig({ ...config, use_trash: v })
                      }
                      label="Use Trash instead of permanent delete"
                    />
                  </div>
                  <Field label="Protection Level">
                    <select
                      value={config.protection_level}
                      onChange={(e) =>
                        setConfig({
                          ...config,
                          protection_level: e.target.value,
                        })
                      }
                      className="w-full px-3 py-1.5 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-solid)] text-xs text-[var(--color-text)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]"
                    >
                      <option value="relaxed">Relaxed</option>
                      <option value="normal">Normal</option>
                      <option value="strict">Strict</option>
                    </select>
                  </Field>
                </Section>

                {/* Appearance */}
                <Section title="Appearance">
                  <div className="flex gap-2">
                    {(['dark', 'light', 'system'] as Theme[]).map((t) => (
                      <button
                        key={t}
                        onClick={() => setTheme(t)}
                        className={`pill flex-1 px-3 py-1.5 text-xs font-medium transition-colors capitalize focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] outline-none ${
                          theme === t
                            ? 'bg-[var(--color-primary)] text-white'
                            : 'text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)]'
                        }`}
                      >
                        {t}
                      </button>
                    ))}
                  </div>
                </Section>

                {/* About */}
                <Section title="About">
                  <div className="space-y-2">
                    <div>
                      <span className="font-medium text-[var(--color-text)]">null-e</span>
                      <p className="text-[11px] text-[var(--color-text-muted)]">Developer Disk Cleanup</p>
                    </div>
                    <p className="text-[11px] text-[var(--color-text-muted)]">License: WTFPL</p>
                    <div>
                      <button
                        onClick={() => setDisclaimerOpen(!disclaimerOpen)}
                        className="text-[11px] text-[var(--color-text-muted)] hover:text-[var(--color-text-secondary)] transition-colors"
                      >
                        Disclaimer {disclaimerOpen ? '\u25BE' : '\u25B8'}
                      </button>
                      {disclaimerOpen && (
                        <div className="mt-2 rounded-xl bg-[var(--color-bg)] p-3 text-[11px] text-[var(--color-text-muted)]">
                          This software is provided &quot;as-is&quot; without warranty of any kind, express or implied. The authors are not responsible for any data loss, corruption, or damage resulting from the use of this software. Use at your own risk. Always maintain backups of important data.
                        </div>
                      )}
                    </div>
                    {/* Check for Updates */}
                    <div className="flex items-center gap-2">
                      {updateStatus === 'idle' && (
                        <button
                          onClick={checkForUpdates}
                          className="flex items-center gap-1.5 text-[11px] text-[var(--color-primary)] hover:underline"
                        >
                          <Download size={11} />
                          Check for updates
                        </button>
                      )}
                      {updateStatus === 'checking' && (
                        <span className="flex items-center gap-1.5 text-[11px] text-[var(--color-text-muted)]">
                          <RefreshCw size={11} className="animate-spin" />
                          Checking...
                        </span>
                      )}
                      {updateStatus === 'none' && (
                        <span className="text-[11px] text-[var(--color-safe)]">
                          You're up to date!
                        </span>
                      )}
                      {updateStatus === 'available' && (
                        <button
                          onClick={installUpdate}
                          className="pill flex items-center gap-1.5 text-[11px] font-medium text-white bg-[var(--color-primary)] px-2.5 py-1 hover:bg-[var(--color-primary-hover)] transition-colors focus-visible:ring-2 focus-visible:ring-white outline-none"
                        >
                          <Download size={11} />
                          Update to v{updateVersion}
                        </button>
                      )}
                      {updateStatus === 'downloading' && (
                        <span className="flex items-center gap-1.5 text-[11px] text-[var(--color-text-muted)]">
                          <RefreshCw size={11} className="animate-spin" />
                          Installing...
                        </span>
                      )}
                    </div>

                    <a
                      href="https://github.com/us/null-e"
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-[11px] text-[var(--color-primary)] hover:underline"
                    >
                      github.com/us/null-e
                    </a>
                  </div>
                </Section>
              </div>
            )}
          </motion.div>
        </>
      )}
    </AnimatePresence>
  );
}

function Section({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div className="bento p-4 space-y-3">
      <h3 className="text-[11px] font-semibold uppercase tracking-wider text-[var(--color-text-muted)]">
        {title}
      </h3>
      <div className="flex flex-col gap-2">{children}</div>
    </div>
  );
}

function Field({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div className="space-y-1.5">
      <label className="text-xs text-[var(--color-text-secondary)]">
        {label}
      </label>
      {children}
    </div>
  );
}

function Toggle({
  checked,
  onChange,
  label,
}: {
  checked: boolean;
  onChange: (v: boolean) => void;
  label?: string;
}) {
  return (
    <button
      onClick={() => onChange(!checked)}
      role="switch"
      aria-checked={checked}
      aria-label={label}
      className={`relative w-9 h-5 rounded-full transition-colors focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] outline-none ${
        checked ? 'bg-[var(--color-safe)]' : 'bg-[var(--color-text-muted)]'
      }`}
    >
      <span
        className={`absolute top-0.5 w-4 h-4 rounded-full bg-white shadow-sm transition-transform ${
          checked ? 'left-[18px]' : 'left-0.5'
        }`}
      />
    </button>
  );
}
