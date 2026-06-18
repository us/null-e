import { useState, useRef, useEffect, useMemo } from 'react';
import { ChevronDown, Zap } from 'lucide-react';
import type { FlatArtifact } from './ResultsView';
import type { SystemEntry } from './SystemItemRow';
import { formatSize } from '@/lib/format';

interface SelectionPresetsProps {
  flatArtifacts: FlatArtifact[];
  systemEntries: SystemEntry[];
  onApply: (paths: Set<string>) => void;
}

interface Preset {
  label: string;
  description: string;
  filter: (artifacts: FlatArtifact[], system: SystemEntry[]) => Set<string>;
}

const SIZE_500MB = 500 * 1024 * 1024;
const SIZE_1GB = 1024 * 1024 * 1024;

const presets: Preset[] = [
  {
    label: '> 500 MB',
    description: 'All items larger than 500 MB',
    filter: (artifacts, system) => {
      const paths = new Set<string>();
      for (const a of artifacts) if (a.artifact.size >= SIZE_500MB) paths.add(a.artifact.path);
      for (const s of system) if (s.size >= SIZE_500MB) paths.add(s.path);
      return paths;
    },
  },
  {
    label: '> 1 GB',
    description: 'All items larger than 1 GB',
    filter: (artifacts, system) => {
      const paths = new Set<string>();
      for (const a of artifacts) if (a.artifact.size >= SIZE_1GB) paths.add(a.artifact.path);
      for (const s of system) if (s.size >= SIZE_1GB) paths.add(s.path);
      return paths;
    },
  },
  {
    label: 'Safe items',
    description: 'System items marked as safe to delete',
    filter: (_artifacts, system) => {
      const paths = new Set<string>();
      for (const s of system) if (s.safetyLevel === 'safe') paths.add(s.path);
      return paths;
    },
  },
  {
    label: 'Build output',
    description: 'All build artifacts (target/, dist/, .next, etc.)',
    filter: (artifacts) => {
      const paths = new Set<string>();
      for (const a of artifacts) if (a.artifact.kind === 'BuildOutput') paths.add(a.artifact.path);
      return paths;
    },
  },
  {
    label: 'All caches',
    description: 'Dependencies + package manager caches',
    filter: (artifacts, system) => {
      const paths = new Set<string>();
      for (const a of artifacts) {
        if (a.artifact.kind === 'Dependencies' || a.artifact.kind === 'Cache')
          paths.add(a.artifact.path);
      }
      for (const s of system) if (s.cacheId) paths.add(s.path);
      return paths;
    },
  },
];

export function SelectionPresets({
  flatArtifacts,
  systemEntries,
  onApply,
}: SelectionPresetsProps) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  // Memoize preset match counts + aggregate sizes so each row is decision-ready (count alone can't
  // tell "> 1 GB (4)" apart from 5 GB vs 40 GB).
  const presetStats = useMemo(() => {
    const sizeByPath = new Map<string, number>();
    for (const a of flatArtifacts) sizeByPath.set(a.artifact.path, a.artifact.size);
    for (const s of systemEntries) sizeByPath.set(s.path, s.size);
    return presets.map((p) => {
      const paths = p.filter(flatArtifacts, systemEntries);
      let size = 0;
      for (const path of paths) size += sizeByPath.get(path) ?? 0;
      return { count: paths.size, size };
    });
  }, [flatArtifacts, systemEntries]);

  // Close on outside click
  useEffect(() => {
    if (!open) return;
    const handleClick = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener('click', handleClick);
    return () => document.removeEventListener('click', handleClick);
  }, [open]);

  // Close on Escape
  useEffect(() => {
    if (!open) return;
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setOpen(false);
    };
    document.addEventListener('keydown', handleKey);
    return () => document.removeEventListener('keydown', handleKey);
  }, [open]);

  return (
    <div className="relative" ref={ref}>
      <button
        onClick={() => setOpen(!open)}
        className="pill shrink-0 flex items-center gap-1 text-xs px-3 py-1.5 bg-transparent text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)] transition-colors focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] outline-none"
        aria-label="Quick select presets"
        aria-expanded={open}
      >
        <Zap size={12} />
        Quick select
        <ChevronDown size={12} />
      </button>

      {open && (
        <div className="absolute bottom-full left-0 mb-1 z-50 w-56 overflow-hidden p-1 rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface-solid)] shadow-xl">
          {presets.map((preset, idx) => {
            const { count: matchCount, size: matchSize } = presetStats[idx];
            return (
              <button
                key={preset.label}
                onClick={() => {
                  onApply(preset.filter(flatArtifacts, systemEntries));
                  setOpen(false);
                }}
                disabled={matchCount === 0}
                className="w-full flex items-center justify-between gap-2 px-3 py-2 rounded-lg text-left hover:bg-[var(--color-surface-hover)] transition-colors disabled:opacity-40 disabled:cursor-not-allowed focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] outline-none"
              >
                <div className="min-w-0">
                  <div className="text-xs font-medium text-[var(--color-text)]">
                    {preset.label}
                  </div>
                  <div className="text-[11px] text-[var(--color-text-secondary)] truncate">
                    {preset.description}
                  </div>
                </div>
                <div className="shrink-0 text-right">
                  <div className="text-[11px] tabular-nums font-medium text-[var(--color-text-secondary)]">
                    {matchCount > 0 ? formatSize(matchSize) : '—'}
                  </div>
                  <div className="text-[10px] tabular-nums text-[var(--color-text-muted)]">
                    {matchCount} item{matchCount === 1 ? '' : 's'}
                  </div>
                </div>
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}
