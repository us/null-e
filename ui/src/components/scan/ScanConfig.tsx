import { useState } from 'react';
import { FolderPlus, X, Play } from 'lucide-react';
import { useScanStore } from '@/stores/scan-store';

export function ScanConfig() {
  const { startScan, isScanning } = useScanStore();
  const [roots, setRoots] = useState<string[]>([]);
  const [maxDepth, setMaxDepth] = useState(10);
  const [minSize, setMinSize] = useState(0);
  const [newRoot, setNewRoot] = useState('');

  const addRoot = () => {
    const trimmed = newRoot.trim();
    if (trimmed && !roots.includes(trimmed)) {
      setRoots([...roots, trimmed]);
      setNewRoot('');
    }
  };

  const removeRoot = (index: number) => {
    setRoots(roots.filter((_, i) => i !== index));
  };

  const handleScan = () => {
    startScan(roots.length > 0 ? roots : []).catch(console.error);
  };

  return (
    <div className="p-4 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] space-y-4">
      <h3 className="text-sm font-semibold text-[var(--color-text)]">Scan Configuration</h3>

      {/* Scan roots */}
      <div className="space-y-2">
        <label className="text-xs text-[var(--color-text-secondary)]">Scan Paths</label>
        <div className="flex gap-2">
          <input
            type="text"
            value={newRoot}
            onChange={(e) => setNewRoot(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && addRoot()}
            placeholder="/Users/you/code"
            className="flex-1 px-3 py-1.5 rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] text-sm text-[var(--color-text)] placeholder:text-[var(--color-text-muted)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]"
          />
          <button
            onClick={addRoot}
            className="px-3 py-1.5 rounded-lg bg-[var(--color-bg-tertiary)] hover:bg-[var(--color-surface-hover)] text-[var(--color-text-secondary)] transition-colors"
          >
            <FolderPlus size={16} />
          </button>
        </div>
        {roots.length > 0 && (
          <div className="flex flex-wrap gap-2">
            {roots.map((root, i) => (
              <span
                key={i}
                className="inline-flex items-center gap-1 px-2 py-1 rounded-md bg-[var(--color-bg-tertiary)] text-xs text-[var(--color-text-secondary)]"
              >
                {root}
                <button onClick={() => removeRoot(i)} className="hover:text-[var(--color-danger)]">
                  <X size={12} />
                </button>
              </span>
            ))}
          </div>
        )}
        {roots.length === 0 && (
          <p className="text-xs text-[var(--color-text-muted)]">
            Leave empty to scan default locations
          </p>
        )}
      </div>

      {/* Max depth */}
      <div className="space-y-1">
        <label className="text-xs text-[var(--color-text-secondary)]">
          Max Depth: {maxDepth}
        </label>
        <input
          type="range"
          min={1}
          max={20}
          value={maxDepth}
          onChange={(e) => setMaxDepth(Number(e.target.value))}
          className="w-full accent-[var(--color-primary)]"
        />
      </div>

      {/* Min size */}
      <div className="space-y-1">
        <label className="text-xs text-[var(--color-text-secondary)]">
          Min Size: {minSize === 0 ? 'No limit' : `${minSize} MB`}
        </label>
        <input
          type="range"
          min={0}
          max={500}
          step={10}
          value={minSize}
          onChange={(e) => setMinSize(Number(e.target.value))}
          className="w-full accent-[var(--color-primary)]"
        />
      </div>

      {/* Scan button */}
      <button
        onClick={handleScan}
        disabled={isScanning}
        className="w-full flex items-center justify-center gap-2 px-4 py-2 rounded-lg bg-[var(--color-primary)] hover:bg-[var(--color-primary-hover)] text-white text-sm font-medium disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
      >
        <Play size={14} />
        <span>Start Scan</span>
      </button>
    </div>
  );
}
