import { useEffect, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X, Save, Loader2, CheckCircle, AlertCircle, Download, RefreshCw } from 'lucide-react';
import { commands } from '@/lib/tauri';
import { useUiStore, type Theme } from '@/stores/ui-store';
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
            initial={{ x: '100%' }}
            animate={{ x: 0 }}
            exit={{ x: '100%' }}
            transition={{ type: 'spring', damping: 30, stiffness: 300 }}
            className="fixed right-0 top-0 bottom-0 w-[380px] max-w-[90vw] z-50 bg-[var(--color-surface-solid)] border-l border-[var(--color-border)] shadow-2xl overflow-y-auto"
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
                  className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium text-white bg-[var(--color-primary)] hover:bg-[var(--color-primary-hover)] disabled:opacity-50 transition-colors"
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
                  className="p-1.5 rounded-lg hover:bg-[var(--color-surface-hover)] text-[var(--color-text-muted)]"
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
              <div className="p-5 space-y-6">
                {error && (
                  <div className="flex items-center gap-2 p-3 rounded-lg bg-red-500/10 text-[var(--color-danger)] text-xs">
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
                        className="flex-1 px-3 py-1.5 rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] text-xs text-[var(--color-text)] placeholder:text-[var(--color-text-muted)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]"
                      />
                      <button
                        onClick={addPath}
                        className="px-3 py-1.5 rounded-lg bg-[var(--color-bg-tertiary)] hover:bg-[var(--color-surface-hover)] text-xs text-[var(--color-text-secondary)] transition-colors"
                      >
                        Add
                      </button>
                    </div>
                    {config.scan_paths.length > 0 ? (
                      <div className="space-y-1">
                        {config.scan_paths.map((path, i) => (
                          <div
                            key={i}
                            className="flex items-center justify-between px-3 py-1.5 rounded-lg bg-[var(--color-bg)]"
                          >
                            <span className="text-[11px] text-[var(--color-text-secondary)] truncate">
                              {path}
                            </span>
                            <button
                              onClick={() => removePath(i)}
                              className="text-[11px] text-[var(--color-danger)] hover:underline shrink-0 ml-2"
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
                      className="w-full px-3 py-1.5 rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] text-xs text-[var(--color-text)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]"
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
                        className={`flex-1 px-3 py-1.5 rounded-lg text-xs font-medium transition-colors capitalize ${
                          theme === t
                            ? 'bg-[var(--color-primary)] text-white'
                            : 'bg-[var(--color-bg-tertiary)] text-[var(--color-text-secondary)] hover:text-[var(--color-text)]'
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
                        <div className="mt-2 rounded-lg bg-[var(--color-bg)] p-3 text-[11px] text-[var(--color-text-muted)]">
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
                          className="flex items-center gap-1.5 text-[11px] font-medium text-white bg-[var(--color-primary)] px-2.5 py-1 rounded-md hover:opacity-90 transition-opacity"
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
    <div className="space-y-3">
      <h3 className="text-[11px] font-semibold uppercase tracking-wider text-[var(--color-text-muted)]">
        {title}
      </h3>
      <div className="space-y-3">{children}</div>
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
}: {
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <button
      onClick={() => onChange(!checked)}
      className={`relative w-9 h-5 rounded-full transition-colors ${
        checked ? 'bg-[var(--color-safe)]' : 'bg-[var(--color-text-muted)]'
      }`}
    >
      <span
        className={`absolute top-0.5 w-4 h-4 rounded-full bg-white transition-transform ${
          checked ? 'left-[18px]' : 'left-0.5'
        }`}
      />
    </button>
  );
}
