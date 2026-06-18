import { useState, useEffect, useMemo, useCallback, useRef } from 'react';
import { Loader2, CheckCircle2, RotateCcw, X } from 'lucide-react';
import { AnimatePresence } from 'framer-motion';
import { useScanStore } from '@/stores/scan-store';
import { useUiStore } from '@/stores/ui-store';
import { useCleanStore } from '@/stores/clean-store';
import { useSystemStore } from '@/stores/system-store';
import {
  events,
  commands,
  type ArtifactDto,
  type CleanFailureDto,
  type CleanSummaryDto,
} from '@/lib/tauri';
import { getTechColor, getKindLabel, TechIcon } from '@/components/shared/TechIcon';
import { fuzzyMatchAny } from '@/lib/fuzzy';
import { formatSize } from '@/lib/format';
import { SearchBar } from '@/components/shared/SearchBar';
import { SelectionPresets } from './SelectionPresets';
import { ActionBar } from './ActionBar';
import { ConfirmDialog } from '@/components/clean/ConfirmDialog';
import { CleaningOverlay } from './CleaningOverlay';
import type { SystemEntry } from './SystemItemRow';

const MIN_AUTO_SELECT_SIZE = 200 * 1024 * 1024; // 200 MB
const SYSTEM_PREFIX = 'system:';

export interface FlatArtifact {
  artifact: ArtifactDto;
  projectName: string;
  projectRoot: string;
  kind: string;
}

export interface KindGroup {
  kind: string;
  label: string;
  color: string;
  items: FlatArtifact[];
  totalSize: number;
  projectCount: number;
}

/** Unified sidebar category: a project-kind group or a system category. */
interface Category {
  id: string;
  label: string;
  color: string;
  iconKind?: string;
  emoji?: string;
  paths: string[];
  totalSize: number;
  count: number;
}

/** A row in the main pane (a project artifact or a system entry). */
interface MainRow {
  path: string;
  name: string;
  detail: string;
  size: number;
  iconKind?: string;
  emoji?: string;
}

interface CacheCleanTarget {
  cacheId: string;
  path: string;
}

interface CacheCleanAggregate {
  totalItems: number;
  succeeded: number;
  failed: number;
  bytesFreed: number;
  failures: CleanFailureDto[];
}

function emptyCacheAggregate(): CacheCleanAggregate {
  return {
    totalItems: 0,
    succeeded: 0,
    failed: 0,
    bytesFreed: 0,
    failures: [],
  };
}

function isTccFailureReason(reason: string): boolean {
  const lower = reason.toLowerCase();
  return lower.includes('operation not permitted')
    || lower.includes('eperm')
    || lower.includes('os error 1');
}

/** Map a side-channel (cache/action) failure reason to a failure category for grouping. */
function categorizeReason(reason: string): string {
  const lower = reason.toLowerCase();
  if (lower.startsWith('refused') || lower.includes('protected location') || lower.includes('not allowed')) {
    return 'refused';
  }
  if (lower.includes('administrator') || lower.includes('needs admin')) return 'needs_admin';
  if (isTccFailureReason(reason)) return 'fda';
  return 'other';
}

function mergeSummaries(summary: CleanSummaryDto, cache: CacheCleanAggregate): CleanSummaryDto {
  if (cache.totalItems === 0) return summary;

  return {
    total_items: summary.total_items + cache.totalItems,
    succeeded: summary.succeeded + cache.succeeded,
    failed: summary.failed + cache.failed,
    // Caches are permanently deleted → their bytes are freed now, never pending.
    bytes_freed: summary.bytes_freed + cache.bytesFreed,
    bytes_pending: summary.bytes_pending,
    used_trash: summary.used_trash,
    method_label: summary.method_label === 'Deleted' ? 'Deleted' : 'Mixed',
    failures: [...summary.failures, ...cache.failures],
  };
}

function createCacheOnlySummary(cache: CacheCleanAggregate): CleanSummaryDto {
  return {
    total_items: cache.totalItems,
    succeeded: cache.succeeded,
    failed: cache.failed,
    bytes_freed: cache.bytesFreed,
    bytes_pending: 0,
    used_trash: false,
    method_label: 'Deleted',
    failures: cache.failures,
  };
}

export function ResultsView() {
  const result = useScanStore((s) => s.result);
  const isScanning = useScanStore((s) => s.isScanning);
  const searchQuery = useUiStore((s) => s.searchQuery);
  const setSearchQuery = useUiStore((s) => s.setSearchQuery);
  const { isCleaning, progress: cleanProgress } = useCleanStore();
  const cleaners = useSystemStore((s) => s.cleaners);
  const caches = useSystemStore((s) => s.caches);
  const skippedCleaners = useSystemStore((s) => s.skippedCleaners);
  const isDetectingSystem = useSystemStore((s) => s.isDetecting);
  const systemError = useSystemStore((s) => s.error);
  const [selectedPaths, setSelectedPaths] = useState<Set<string>>(new Set());
  // Surfaces the silent ≥200 MB pre-selection so the user knows why the action bar is already armed.
  const [autoSelectInfo, setAutoSelectInfo] = useState<{ count: number; size: number } | null>(null);
  const [autoNoteDismissed, setAutoNoteDismissed] = useState(false);
  // Active sidebar category filter — null = "All items" (flat heat-list sorted by size).
  const [activeCategory, setActiveCategory] = useState<string | null>(null);
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [diskUsed, setDiskUsed] = useState<number | undefined>();
  const [diskTotal, setDiskTotal] = useState<number | undefined>();
  const cacheCleanPromiseRef = useRef<Promise<CacheCleanAggregate> | null>(null);
  const autoSelectDone = useRef(false);

  // Fetch disk info once on mount
  useEffect(() => {
    commands.getDiskInfo().then((info) => {
      setDiskTotal(info.total);
      // On macOS APFS, info.used only reports the root volume.
      // Compute real used = total - available for accurate disk usage.
      setDiskUsed(info.total - info.available);
    }).catch(console.error);
  }, []);

  // Listen for clean events
  useEffect(() => {
    let cancelled = false;
    const unlisteners: Array<() => void> = [];
    const setup = async () => {
      const unProgress = await events.onCleanProgress((payload) => {
        useCleanStore.getState().setProgress(payload);
      });
      if (cancelled) { unProgress(); return; }
      unlisteners.push(unProgress);

      const unComplete = await events.onCleanComplete((payload) => {
        void (async () => {
          const cacheAggregate = cacheCleanPromiseRef.current
            ? await cacheCleanPromiseRef.current
            : emptyCacheAggregate();
          cacheCleanPromiseRef.current = null;
          useCleanStore.getState().setSummary(mergeSummaries(payload, cacheAggregate));
          useUiStore.getState().setAppState('done');
          await useSystemStore.getState().detectSystem();
        })().catch((err) => {
          useCleanStore.getState().setError(
            err instanceof Error ? err.message : String(err)
          );
          useUiStore.getState().setAppState('results');
        });
      });
      if (cancelled) { unComplete(); return; }
      unlisteners.push(unComplete);
    };
    setup().catch(console.error);
    return () => {
      cancelled = true;
      unlisteners.forEach((fn) => fn());
    };
  }, []);

  // Flatten all artifacts
  const flatArtifacts = useMemo<FlatArtifact[]>(() => {
    if (!result) return [];
    const items: FlatArtifact[] = [];
    for (const project of result.projects) {
      for (const artifact of project.artifacts) {
        items.push({
          artifact,
          projectName: project.name,
          projectRoot: project.root,
          kind: project.kind,
        });
      }
    }
    return items;
  }, [result]);

  // Normalize system items into SystemEntry[]
  const systemEntries = useMemo<SystemEntry[]>(() => {
    const entries: SystemEntry[] = [];
    for (const c of cleaners) {
      // Keep items that occupy space, OR that expose a reclaim action / are OS-managed even at
      // size 0 (e.g. Time Machine local snapshots whose size is unknowable but are reclaimable).
      const keep =
        c.size > 0 ||
        !!c.clean_command ||
        c.reclaimability === 'os_managed_purgeable';
      if (keep) {
        entries.push({
          path: `${SYSTEM_PREFIX}${c.path}`,
          realPath: c.path,
          name: c.name,
          size: c.size,
          icon: c.icon,
          description: c.description,
          category: c.category,
          safetyLevel: c.safety_level,
          reclaimability: c.reclaimability,
          reclaimableBytes: c.reclaimable_bytes,
          hasCleanCommand: !!c.clean_command,
        });
      }
    }
    for (const c of caches) {
      if (c.size > 0) {
        entries.push({
          path: `${SYSTEM_PREFIX}${c.path}`,
          realPath: c.path,
          name: c.name,
          size: c.size,
          icon: c.icon,
          description: c.description,
          category: 'Package Caches',
          cacheId: c.id,
          reclaimability: 'user_reclaimable',
          reclaimableBytes: c.size,
        });
      }
    }
    return entries;
  }, [cleaners, caches]);

  // Search filter
  const query = searchQuery.toLowerCase().trim();

  const filteredArtifacts = useMemo(() => {
    if (!query) return flatArtifacts;
    return flatArtifacts.filter((a) =>
      fuzzyMatchAny(query, [
        a.projectName,
        a.artifact.name,
        a.artifact.path,
        getKindLabel(a.kind),
      ])
    );
  }, [flatArtifacts, query]);

  const filteredSystemEntries = useMemo(() => {
    if (!query) return systemEntries;
    return systemEntries.filter((e) =>
      fuzzyMatchAny(query, [e.name, e.category, e.description, e.realPath])
    );
  }, [systemEntries, query]);

  // Group by kind (from filtered artifacts)
  const groups = useMemo<KindGroup[]>(() => {
    const map = new Map<string, FlatArtifact[]>();
    const projectSets = new Map<string, Set<string>>();
    for (const item of filteredArtifacts) {
      const label = getKindLabel(item.kind);
      if (!map.has(label)) {
        map.set(label, []);
        projectSets.set(label, new Set());
      }
      map.get(label)!.push(item);
      projectSets.get(label)!.add(item.projectRoot);
    }
    const result: KindGroup[] = [];
    for (const [label, items] of map) {
      items.sort((a, b) => b.artifact.size - a.artifact.size);
      const totalSize = items.reduce((sum, i) => sum + i.artifact.size, 0);
      result.push({
        kind: items[0].kind,
        label,
        color: getTechColor(items[0].kind),
        items,
        totalSize,
        projectCount: projectSets.get(label)!.size,
      });
    }
    return result.sort((a, b) => b.totalSize - a.totalSize);
  }, [filteredArtifacts]);

  // Smart auto-select: project artifacts >= 200 MB (only on first load)
  useEffect(() => {
    if (!result || autoSelectDone.current) return;
    autoSelectDone.current = true;
    const autoSelected = new Set<string>();
    let autoSize = 0;
    for (const project of result.projects) {
      for (const artifact of project.artifacts) {
        if (artifact.size >= MIN_AUTO_SELECT_SIZE) {
          autoSelected.add(artifact.path);
          autoSize += artifact.size;
        }
      }
    }
    setSelectedPaths(autoSelected);
    setAutoSelectInfo(autoSelected.size > 0 ? { count: autoSelected.size, size: autoSize } : null);
  }, [result]);

  const togglePath = useCallback((path: string) => {
    setSelectedPaths((prev) => {
      const next = new Set(prev);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });
  }, []);

  // Toggle selection of an entire category's paths (sidebar checkbox).
  const toggleCategory = useCallback((paths: string[]) => {
    setSelectedPaths((prev) => {
      const next = new Set(prev);
      const allSel = paths.every((p) => next.has(p));
      if (allSel) paths.forEach((p) => next.delete(p));
      else paths.forEach((p) => next.add(p));
      return next;
    });
  }, []);

  const totalItemCount = flatArtifacts.length + systemEntries.length;

  const selectAll = useCallback(() => {
    const all = new Set<string>();
    for (const { artifact } of flatArtifacts) all.add(artifact.path);
    for (const entry of systemEntries) all.add(entry.path);
    setSelectedPaths(all);
  }, [flatArtifacts, systemEntries]);

  const deselectAll = useCallback(() => {
    setSelectedPaths(new Set());
  }, []);

  const applyPreset = useCallback((paths: Set<string>) => {
    setSelectedPaths(paths);
  }, []);

  const allSelected = useMemo(() => {
    if (totalItemCount === 0) return false;
    for (const { artifact } of flatArtifacts) {
      if (!selectedPaths.has(artifact.path)) return false;
    }
    for (const entry of systemEntries) {
      if (!selectedPaths.has(entry.path)) return false;
    }
    return true;
  }, [flatArtifacts, systemEntries, selectedPaths, totalItemCount]);

  const selectedSize = useMemo(() => {
    let total = 0;
    for (const { artifact } of flatArtifacts) {
      if (selectedPaths.has(artifact.path)) total += artifact.size;
    }
    for (const entry of systemEntries) {
      if (selectedPaths.has(entry.path)) total += entry.size;
    }
    return total;
  }, [flatArtifacts, systemEntries, selectedPaths]);

  const totalCleanable = useMemo(() => {
    const projectTotal = result?.total_cleanable ?? 0;
    const systemTotal = systemEntries.reduce((sum, e) => sum + e.size, 0);
    return projectTotal + systemTotal;
  }, [result, systemEntries]);

  // ─── Sidebar category model: project kinds + system categories, unified ───
  const categories = useMemo<Category[]>(() => {
    const out: Category[] = [];
    for (const g of groups) {
      out.push({
        id: `kind:${g.label}`,
        label: g.label,
        color: g.color,
        iconKind: g.kind,
        paths: g.items.map((i) => i.artifact.path),
        totalSize: g.totalSize,
        count: g.items.length,
      });
    }
    const sys = new Map<string, SystemEntry[]>();
    for (const e of filteredSystemEntries) {
      const list = sys.get(e.category) ?? [];
      list.push(e);
      sys.set(e.category, list);
    }
    for (const [cat, entries] of sys) {
      out.push({
        id: `sys:${cat}`,
        label: cat,
        color: 'var(--color-lav)',
        emoji: entries[0].icon,
        paths: entries.map((e) => e.path),
        totalSize: entries.reduce((s, e) => s + e.size, 0),
        count: entries.length,
      });
    }
    return out.sort((a, b) => b.totalSize - a.totalSize);
  }, [groups, filteredSystemEntries]);

  const maxCatSize = useMemo(
    () => categories.reduce((m, c) => Math.max(m, c.totalSize), 0),
    [categories]
  );

  // Rows for the main pane: all items (activeCategory === null) or one category's items,
  // always sorted by size so the biggest hogs surface first (no endless scroll).
  const mainRows = useMemo<MainRow[]>(() => {
    const rows: MainRow[] = [];
    const wantKind = activeCategory?.startsWith('kind:') ? activeCategory.slice(5) : null;
    const wantSys = activeCategory?.startsWith('sys:') ? activeCategory.slice(4) : null;
    if (activeCategory === null || wantKind !== null) {
      for (const a of filteredArtifacts) {
        const label = getKindLabel(a.kind);
        if (wantKind !== null && label !== wantKind) continue;
        rows.push({
          path: a.artifact.path,
          name: a.projectName,
          detail: `${label} · ${a.artifact.name}`,
          size: a.artifact.size,
          iconKind: a.kind,
        });
      }
    }
    if (activeCategory === null || wantSys !== null) {
      for (const e of filteredSystemEntries) {
        if (wantSys !== null && e.category !== wantSys) continue;
        rows.push({ path: e.path, name: e.name, detail: e.category, size: e.size, emoji: e.icon });
      }
    }
    return rows.sort((a, b) => b.size - a.size);
  }, [activeCategory, filteredArtifacts, filteredSystemEntries]);

  const maxRowSize = useMemo(
    () => mainRows.reduce((m, r) => Math.max(m, r.size), 0),
    [mainRows]
  );

  // Donut: category proportions of total cleanable — an at-a-glance "where is the space".
  const donutGradient = useMemo(() => {
    const total = categories.reduce((s, c) => s + c.totalSize, 0) || 1;
    let acc = 0;
    const stops: string[] = [];
    for (const c of categories.slice(0, 6)) {
      const from = (acc / total) * 100;
      acc += c.totalSize;
      const to = (acc / total) * 100;
      stops.push(`${c.color} ${from}% ${to}%`);
    }
    stops.push(`var(--color-bg-tertiary) ${(acc / total) * 100}% 100%`);
    return `conic-gradient(${stops.join(', ')})`;
  }, [categories]);

  const selectedCountIn = useCallback(
    (paths: string[]) => paths.reduce((n, p) => (selectedPaths.has(p) ? n + 1 : n), 0),
    [selectedPaths]
  );

  const activeCat = activeCategory
    ? categories.find((c) => c.id === activeCategory) ?? null
    : null;

  const cacheKeyToId = useMemo(() => {
    const map = new Map<string, string>();
    for (const entry of systemEntries) {
      if (entry.cacheId) map.set(entry.path, entry.cacheId);
    }
    return map;
  }, [systemEntries]);

  const systemKeys = useMemo(
    () => new Set(systemEntries.map((e) => e.path)),
    [systemEntries]
  );

  const systemEntryByKey = useMemo(() => {
    const map = new Map<string, SystemEntry>();
    for (const entry of systemEntries) map.set(entry.path, entry);
    return map;
  }, [systemEntries]);

  const handleClean = async (useTrash: boolean) => {
    setConfirmOpen(false);
    useUiStore.getState().setAppState('cleaning');
    useCleanStore.setState({
      isCleaning: true,
      progress: null,
      summary: null,
      error: null,
    });

    const artifactPaths: string[] = [];
    const systemPathsToDelete: string[] = [];
    const cacheTargetsToClean: CacheCleanTarget[] = [];
    // System items that must NOT go through a generic recursive delete — they run a typed,
    // backend-resolved action (official command, e.g. Time Machine snapshot thinning; or a
    // guarded delete). This is what stops "/" or a snapshot item being rm-rf'd.
    const actionTargets: Array<{ realPath: string; name: string }> = [];

    for (const key of selectedPaths) {
      if (systemKeys.has(key)) {
        const cacheId = cacheKeyToId.get(key);
        const entry = systemEntryByKey.get(key);
        const realPath = key.slice(SYSTEM_PREFIX.length);
        if (cacheId) {
          cacheTargetsToClean.push({ cacheId, path: realPath });
        } else if (
          entry &&
          (entry.hasCleanCommand || (entry.reclaimability && entry.reclaimability !== 'user_reclaimable'))
        ) {
          // Has an official command, or is OS-managed/needs-admin → typed server-side action.
          actionTargets.push({ realPath: entry.realPath ?? realPath, name: entry.name });
        } else {
          systemPathsToDelete.push(realPath);
        }
      } else {
        artifactPaths.push(key);
      }
    }

    const allPathsToDelete = [...artifactPaths, ...systemPathsToDelete];
    const hasPathClean = allPathsToDelete.length > 0;
    const hasSideChannel = cacheTargetsToClean.length > 0 || actionTargets.length > 0;

    const cacheSummaryPromise = hasSideChannel
      ? Promise.allSettled([
          ...cacheTargetsToClean.map(async (target) => ({
            path: target.path,
            bytesFreed: await commands.cleanCache(target.cacheId),
          })),
          ...actionTargets.map(async (target) => {
            const res = await commands.runSystemAction(target.realPath, target.name);
            return { path: target.realPath, bytesFreed: res.bytes_freed };
          }),
        ]).then((results): CacheCleanAggregate => {
          const aggregate = emptyCacheAggregate();
          aggregate.totalItems = cacheTargetsToClean.length + actionTargets.length;
          const labels = [
            ...cacheTargetsToClean.map((t) => t.path),
            ...actionTargets.map((t) => t.realPath),
          ];

          results.forEach((result, index) => {
            if (result.status === 'fulfilled') {
              aggregate.succeeded += 1;
              aggregate.bytesFreed += result.value.bytesFreed;
              return;
            }

            const reason = result.reason instanceof Error
              ? result.reason.message
              : String(result.reason);
            aggregate.failed += 1;
            aggregate.failures.push({
              path: labels[index] ?? '',
              reason,
              is_tcc: isTccFailureReason(reason),
              category: categorizeReason(reason),
            });
          });

          return aggregate;
        })
      : Promise.resolve(emptyCacheAggregate());

    cacheCleanPromiseRef.current = cacheSummaryPromise;

    if (hasPathClean) {
      await useCleanStore.getState().startClean(allPathsToDelete, {
        use_trash: useTrash,
        dry_run: false,
        force: false,
      });

      if (useCleanStore.getState().error) {
        const cacheSummary = await cacheSummaryPromise;
        cacheCleanPromiseRef.current = null;

        if (cacheSummary.totalItems > 0) {
          useCleanStore.getState().setSummary(createCacheOnlySummary(cacheSummary));
          useUiStore.getState().setAppState('done');
          await useSystemStore.getState().detectSystem();
        } else {
          useUiStore.getState().setAppState('results');
        }
        return;
      }
    }

    if (!hasPathClean && hasSideChannel) {
      const cacheSummary = await cacheSummaryPromise;
      cacheCleanPromiseRef.current = null;
      useCleanStore.getState().setSummary(createCacheOnlySummary(cacheSummary));
      useUiStore.getState().setAppState('done');
      await useSystemStore.getState().detectSystem();
    }
  };

  if (!result) return null;

  // Affirmative empty state: a finished scan that genuinely found nothing should celebrate a clean
  // disk, not leave the user on a blank list that reads as broken. Only when there's truly nothing
  // (no search filtering it out) and no work is still in flight.
  const nothingFound =
    flatArtifacts.length === 0 &&
    systemEntries.length === 0 &&
    !isScanning &&
    !isDetectingSystem &&
    query === '';

  const handleScanAgain = () => {
    useScanStore.getState().reset();
    useUiStore.getState().setAppState('welcome');
  };

  if (nothingFound) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-6 px-6 text-center">
        <div className="flex items-center justify-center w-16 h-16 rounded-full bg-[var(--color-safe)]/10">
          <CheckCircle2 size={32} className="text-[var(--color-safe)]" />
        </div>
        <div>
          <h2 className="text-lg font-semibold text-[var(--color-text)]">
            Nothing to clean — you're all set
          </h2>
          <p className="mt-2 max-w-sm text-sm text-[var(--color-text-secondary)]">
            null-e found no reclaimable build artifacts or caches in the scanned locations. Your
            disk is already tidy.
          </p>
          {diskTotal !== undefined && diskUsed !== undefined && (
            <p className="mt-3 text-sm text-[var(--color-text-muted)] tabular-nums">
              {formatSize(diskTotal - diskUsed)} free of {formatSize(diskTotal)}
            </p>
          )}
        </div>
        <button
          onClick={handleScanAgain}
          className="inline-flex items-center gap-2 rounded-xl bg-[var(--color-primary)] px-5 py-2.5 text-sm font-medium text-white transition-colors hover:bg-[var(--color-primary-hover)] focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] outline-none"
        >
          <RotateCcw size={15} />
          Scan again
        </button>
        <button
          onClick={() => useUiStore.getState().setSettingsOpen(true)}
          className="text-xs text-[var(--color-text-muted)] hover:text-[var(--color-text-secondary)] transition-colors"
        >
          Change scan paths
        </button>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      <div className="flex flex-1 min-h-0">
        {/* ─── Sidebar: donut + category overview (at-a-glance, no scroll) ─── */}
        <aside className="w-[268px] shrink-0 flex flex-col border-r border-[var(--color-border)] bg-[var(--color-surface)]">
          {/* Donut + totals */}
          <div className="p-4 border-b border-[var(--color-border)] flex items-center gap-3.5">
            <div className="relative w-[78px] h-[78px] shrink-0">
              <div className="w-full h-full rounded-full" style={{ background: donutGradient }} />
              <div className="absolute inset-[13px] rounded-full bg-[var(--color-surface-solid)] flex items-center justify-center">
                <span className="display text-base text-[var(--color-primary)] tabular-nums leading-none whitespace-nowrap">
                  {totalCleanable > 0 ? Math.round((selectedSize / totalCleanable) * 100) : 0}%
                </span>
              </div>
            </div>
            <div className="min-w-0">
              <div className="text-[11px] uppercase tracking-wider text-[var(--color-text-muted)]">Selected</div>
              <div className="text-sm text-[var(--color-text-secondary)]">of {formatSize(totalCleanable)}</div>
              {diskUsed !== undefined && diskTotal !== undefined && (
                <div className="mt-1 text-[11px] text-[var(--color-text-muted)] tabular-nums">
                  {formatSize(diskTotal - diskUsed)} free
                </div>
              )}
            </div>
          </div>

          {/* Category list with proportion bars */}
          <div className="flex-1 overflow-y-auto p-2 space-y-0.5">
            <button
              onClick={() => setActiveCategory(null)}
              className={`w-full flex items-center gap-2.5 px-2.5 py-2 rounded-xl text-left transition-colors focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] outline-none ${activeCategory === null ? 'bg-[var(--color-primary-soft)]' : 'hover:bg-[var(--color-surface-hover)]'}`}
            >
              <span className="flex items-center justify-center w-7 h-7 rounded-lg bg-[var(--color-bg-tertiary)] text-[13px]">▦</span>
              <span className="flex-1 text-sm font-medium">All items</span>
              <span className="text-xs tabular-nums text-[var(--color-text-muted)]">{formatSize(totalCleanable)}</span>
            </button>

            {categories.map((cat) => {
              const sel = selectedCountIn(cat.paths);
              const allSel = sel === cat.paths.length && cat.paths.length > 0;
              const isActive = activeCategory === cat.id;
              const pct = maxCatSize > 0 ? (cat.totalSize / maxCatSize) * 100 : 0;
              return (
                <div
                  key={cat.id}
                  onClick={() => setActiveCategory(cat.id)}
                  className={`flex items-center gap-2 px-2.5 py-2 rounded-xl cursor-pointer transition-colors ${isActive ? 'bg-[var(--color-primary-soft)]' : 'hover:bg-[var(--color-surface-hover)]'}`}
                >
                  <button
                    onClick={(e) => { e.stopPropagation(); toggleCategory(cat.paths); }}
                    aria-label={`Select all ${cat.label}`}
                    className={`w-4 h-4 rounded-[5px] border-[1.5px] border-[var(--color-primary)] shrink-0 focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] outline-none ${allSel ? 'bg-[var(--color-primary)]' : sel > 0 ? 'bg-[var(--color-primary)]/40' : ''}`}
                  />
                  <span
                    className="flex items-center justify-center w-7 h-7 rounded-lg shrink-0"
                    style={{ backgroundColor: `color-mix(in srgb, ${cat.color} 16%, transparent)` }}
                  >
                    {cat.iconKind ? <TechIcon kind={cat.iconKind} size={16} /> : <span className="text-[13px]">{cat.emoji}</span>}
                  </span>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center justify-between gap-2">
                      <span className="text-[13px] font-medium truncate">{cat.label}</span>
                      <span className="text-xs tabular-nums font-semibold shrink-0" style={{ color: cat.color }}>{formatSize(cat.totalSize)}</span>
                    </div>
                    <div className="mt-1 h-1 rounded-full bg-[var(--color-bg-tertiary)] overflow-hidden">
                      <div className="h-full rounded-full" style={{ width: `${pct}%`, backgroundColor: cat.color }} />
                    </div>
                  </div>
                </div>
              );
            })}

            {isDetectingSystem && (
              <div className="flex items-center gap-2 px-2.5 py-2 text-xs text-[var(--color-text-muted)]">
                <Loader2 size={12} className="animate-spin" /> Detecting system…
              </div>
            )}
          </div>

          {/* Presets + select-all */}
          <div className="p-2.5 border-t border-[var(--color-border)] flex items-center gap-2">
            <SelectionPresets flatArtifacts={flatArtifacts} systemEntries={systemEntries} onApply={applyPreset} />
            <button
              onClick={allSelected ? deselectAll : selectAll}
              className="pill px-3 py-1.5 text-xs text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)] transition-colors focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] outline-none"
            >
              {allSelected ? 'Clear' : 'Select all'}
            </button>
          </div>
        </aside>

        {/* ─── Main: the active category's items (drill-down), biggest first ─── */}
        <div className="flex-1 flex flex-col min-w-0">
          <div className="flex items-center gap-3 px-4 py-2.5 border-b border-[var(--color-border)]">
            <h3 className="text-sm font-semibold flex items-center gap-2 shrink-0">
              {activeCat ? (
                <>
                  <span
                    className="flex items-center justify-center w-6 h-6 rounded-lg"
                    style={{ backgroundColor: `color-mix(in srgb, ${activeCat.color} 16%, transparent)` }}
                  >
                    {activeCat.iconKind ? <TechIcon kind={activeCat.iconKind} size={14} /> : <span className="text-xs">{activeCat.emoji}</span>}
                  </span>
                  {activeCat.label}
                </>
              ) : 'All items'}
              <span className="text-xs font-normal text-[var(--color-text-muted)] tabular-nums">{mainRows.length}</span>
            </h3>
            <div className="flex-1 max-w-xs">
              <SearchBar value={searchQuery} onChange={setSearchQuery} placeholder="Filter items…" />
            </div>
            {isScanning && (
              <span className="flex items-center gap-1.5 text-xs text-[var(--color-text-muted)] ml-auto">
                <Loader2 size={12} className="animate-spin" /> Updating…
              </span>
            )}
          </div>

          {/* Auto-select note + explainer — only in the All view to avoid clutter */}
          {activeCategory === null && autoSelectInfo && !autoNoteDismissed && selectedPaths.size > 0 && (
            <div className="mx-3 mt-3 flex items-start gap-2 rounded-xl border border-[var(--color-primary)]/25 bg-[var(--color-primary)]/10 px-4 py-2.5 text-xs text-[var(--color-text-secondary)]">
              <CheckCircle2 size={14} className="mt-0.5 shrink-0 text-[var(--color-primary)]" />
              <p className="flex-1">
                <span className="font-semibold text-[var(--color-text)]">Pre-selected {autoSelectInfo.count} large item{autoSelectInfo.count === 1 ? '' : 's'} ({formatSize(autoSelectInfo.size)})</span>{' '}
                over 200&nbsp;MB to get you started — review and adjust before cleaning.
              </p>
              <button onClick={() => setAutoNoteDismissed(true)} aria-label="Dismiss auto-selection note" className="shrink-0 rounded p-0.5 text-[var(--color-text-muted)] hover:text-[var(--color-text)] focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] outline-none"><X size={13} /></button>
            </div>
          )}
          {activeCategory === null && (systemEntries.some((e) => e.reclaimability === 'os_managed_purgeable' || e.reclaimability === 'sip_protected') || skippedCleaners.length > 0) && (
            <div className="mx-3 mt-3 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)]/60 px-4 py-3 text-xs text-[var(--color-text-secondary)]">
              {systemEntries.some((e) => e.reclaimability === 'os_managed_purgeable') && (
                <p><span className="font-semibold text-[var(--color-text)]">Some space is OS-managed.</span> macOS keeps purgeable space and local snapshots no app can delete directly — null-e can ask macOS to thin snapshots, but the amount is decided by macOS.</p>
              )}
              {skippedCleaners.length > 0 && (
                <p className="mt-1.5"><span className="font-semibold text-[var(--color-text)]">{skippedCleaners.length} check{skippedCleaners.length === 1 ? '' : 's'} skipped:</span> {skippedCleaners.join(' · ')} — usually a permission issue (enable Full Disk Access).</p>
              )}
            </div>
          )}
          {systemError && (
            <div className="mx-3 mt-3 flex items-center gap-2 px-4 py-2 rounded-xl bg-[var(--color-danger)]/10 text-xs text-[var(--color-danger)]">
              System detection failed: {systemError}
            </div>
          )}

          {/* Item rows — flat, biggest first */}
          <div className="flex-1 overflow-y-auto px-2 py-2">
            {mainRows.map((row) => {
              const isSel = selectedPaths.has(row.path);
              const color = row.iconKind ? getTechColor(row.iconKind) : 'var(--color-lav)';
              const pct = maxRowSize > 0 ? (row.size / maxRowSize) * 100 : 0;
              return (
                <button
                  key={row.path}
                  onClick={() => togglePath(row.path)}
                  className={`w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-left transition-colors focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] outline-none ${isSel ? 'bg-[var(--color-primary-soft)]' : 'hover:bg-[var(--color-surface-hover)]'}`}
                >
                  <span className={`w-4 h-4 rounded-[5px] border-[1.5px] border-[var(--color-primary)] shrink-0 ${isSel ? 'bg-[var(--color-primary)]' : ''}`} />
                  <span
                    className="flex items-center justify-center w-8 h-8 rounded-xl shrink-0"
                    style={{ backgroundColor: `color-mix(in srgb, ${color} 15%, transparent)` }}
                  >
                    {row.iconKind ? <TechIcon kind={row.iconKind} size={18} /> : <span className="text-base">{row.emoji}</span>}
                  </span>
                  <div className="flex-1 min-w-0">
                    <div className="text-sm font-medium truncate">{row.name}</div>
                    <div className="text-xs text-[var(--color-text-muted)] truncate">{row.detail}</div>
                  </div>
                  <div className="w-24 h-1.5 rounded-full bg-[var(--color-bg-tertiary)] overflow-hidden shrink-0 hidden sm:block">
                    <div className="h-full rounded-full" style={{ width: `${pct}%`, backgroundColor: color }} />
                  </div>
                  <span className="text-sm font-bold tabular-nums shrink-0 w-[78px] text-right" style={{ color }}>{formatSize(row.size)}</span>
                </button>
              );
            })}

            {mainRows.length === 0 && !isDetectingSystem && (
              <div className="flex items-center justify-center h-full text-sm text-[var(--color-text-muted)]">
                {query ? 'No items match your search' : 'No items in this category'}
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Sticky action bar */}
      <AnimatePresence>
        {selectedPaths.size > 0 && (
          <ActionBar
            selectedSize={selectedSize}
            selectedCount={selectedPaths.size}
            onClean={() => setConfirmOpen(true)}
          />
        )}
      </AnimatePresence>

      {/* Confirm dialog */}
      <ConfirmDialog
        open={confirmOpen}
        onClose={() => setConfirmOpen(false)}
        onConfirm={handleClean}
        selectedPaths={Array.from(selectedPaths)}
        totalSize={selectedSize}
      />

      {/* Cleaning overlay */}
      {isCleaning && cleanProgress && (
        <CleaningOverlay progress={cleanProgress} />
      )}
    </div>
  );
}
