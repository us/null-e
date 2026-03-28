import { useState, useEffect, useMemo, useCallback } from 'react';
import { Monitor, Loader2 } from 'lucide-react';
import { AnimatePresence } from 'framer-motion';
import { useScanStore } from '@/stores/scan-store';
import { useUiStore } from '@/stores/ui-store';
import { useCleanStore } from '@/stores/clean-store';
import { useSystemStore } from '@/stores/system-store';
import { events, commands, type ArtifactDto } from '@/lib/tauri';
import { getTechColor, getKindLabel } from '@/components/shared/TechIcon';
import { TechIcon } from '@/components/shared/TechIcon';
import { DiskBar } from './DiskBar';
import { ImpactCards, type ImpactItem } from './ImpactCards';
import { ViewToolbar } from './ViewToolbar';
import { SelectionPresets } from './SelectionPresets';
import { GroupedList } from './GroupedList';
import { FlatList } from './FlatList';
import { SystemSection } from './SystemSection';
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

export function ResultsView() {
  const result = useScanStore((s) => s.result);
  const viewMode = useUiStore((s) => s.viewMode);
  const searchQuery = useUiStore((s) => s.searchQuery);
  const { isCleaning, progress: cleanProgress } = useCleanStore();
  const cleaners = useSystemStore((s) => s.cleaners);
  const caches = useSystemStore((s) => s.caches);
  const isDetectingSystem = useSystemStore((s) => s.isDetecting);
  const systemError = useSystemStore((s) => s.error);
  const [selectedPaths, setSelectedPaths] = useState<Set<string>>(new Set());
  const [collapsedGroups, setCollapsedGroups] = useState<Set<string>>(new Set());
  const [groupsInitialized, setGroupsInitialized] = useState(false);
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [diskUsed, setDiskUsed] = useState<number | undefined>();
  const [diskTotal, setDiskTotal] = useState<number | undefined>();

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
        useCleanStore.getState().setSummary(payload);
        useUiStore.getState().setAppState('done');
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
      if (c.size > 0) {
        entries.push({
          path: `${SYSTEM_PREFIX}${c.path}`,
          name: c.name,
          size: c.size,
          icon: c.icon,
          description: c.description,
          category: c.category,
          safetyLevel: c.safety_level,
        });
      }
    }
    for (const c of caches) {
      if (c.size > 0) {
        entries.push({
          path: `${SYSTEM_PREFIX}${c.path}`,
          name: c.name,
          size: c.size,
          icon: c.icon,
          description: c.description,
          category: 'Package Caches',
          cacheId: c.id,
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
      a.projectName.toLowerCase().includes(query) ||
      a.artifact.name.toLowerCase().includes(query) ||
      a.artifact.path.toLowerCase().includes(query) ||
      getKindLabel(a.kind).toLowerCase().includes(query)
    );
  }, [flatArtifacts, query]);

  const filteredSystemEntries = useMemo(() => {
    if (!query) return systemEntries;
    return systemEntries.filter((e) =>
      e.name.toLowerCase().includes(query) ||
      e.category.toLowerCase().includes(query) ||
      e.description.toLowerCase().includes(query)
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

  // Default: top 2 groups by size open, rest collapsed
  useEffect(() => {
    if (groups.length > 0 && !groupsInitialized) {
      const topLabels = new Set(groups.slice(0, 2).map((g) => g.label));
      setCollapsedGroups(new Set(groups.filter((g) => !topLabels.has(g.label)).map((g) => g.label)));
      setGroupsInitialized(true);
    }
  }, [groups, groupsInitialized]);

  // System category count for toolbar
  const systemCategoryCount = useMemo(
    () => new Set(filteredSystemEntries.map((e) => e.category)).size,
    [filteredSystemEntries]
  );

  // Top-3 impact items (from unfiltered data)
  const impactItems = useMemo<ImpactItem[]>(() => {
    const all: { path: string; name: string; detail: string; size: number; icon: React.ReactNode }[] = [];
    for (const a of flatArtifacts) {
      all.push({
        path: a.artifact.path,
        name: a.projectName,
        detail: `${getKindLabel(a.kind)} · ${a.artifact.name}`,
        size: a.artifact.size,
        icon: <TechIcon kind={a.kind} size={18} />,
      });
    }
    for (const e of systemEntries) {
      all.push({
        path: e.path,
        name: e.name,
        detail: e.category,
        size: e.size,
        icon: <span className="text-base">{e.icon}</span>,
      });
    }
    all.sort((a, b) => b.size - a.size);
    return all.slice(0, 3);
  }, [flatArtifacts, systemEntries]);

  // Max size across all items for relative bars
  const maxSize = useMemo(() => {
    let max = 0;
    for (const { artifact } of flatArtifacts) {
      if (artifact.size > max) max = artifact.size;
    }
    for (const entry of systemEntries) {
      if (entry.size > max) max = entry.size;
    }
    return max;
  }, [flatArtifacts, systemEntries]);

  // Smart auto-select: project artifacts >= 200 MB
  useEffect(() => {
    if (!result) return;
    const autoSelected = new Set<string>();
    for (const project of result.projects) {
      for (const artifact of project.artifacts) {
        if (artifact.size >= MIN_AUTO_SELECT_SIZE) {
          autoSelected.add(artifact.path);
        }
      }
    }
    setSelectedPaths(autoSelected);
  }, [result]);

  const togglePath = useCallback((path: string) => {
    setSelectedPaths((prev) => {
      const next = new Set(prev);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });
  }, []);

  const toggleGroup = useCallback((group: KindGroup) => {
    setSelectedPaths((prev) => {
      const next = new Set(prev);
      const paths = group.items.map((i) => i.artifact.path);
      const allSelected = paths.every((p) => next.has(p));
      if (allSelected) {
        paths.forEach((p) => next.delete(p));
      } else {
        paths.forEach((p) => next.add(p));
      }
      return next;
    });
  }, []);

  const toggleSystemCategory = useCallback((paths: string[]) => {
    setSelectedPaths((prev) => {
      const next = new Set(prev);
      const allSelected = paths.every((p) => next.has(p));
      if (allSelected) {
        paths.forEach((p) => next.delete(p));
      } else {
        paths.forEach((p) => next.add(p));
      }
      return next;
    });
  }, []);

  const toggleCollapse = useCallback((label: string) => {
    setCollapsedGroups((prev) => {
      const next = new Set(prev);
      if (next.has(label)) next.delete(label);
      else next.add(label);
      return next;
    });
  }, []);

  const totalItemCount = flatArtifacts.length + systemEntries.length;
  const filteredItemCount = filteredArtifacts.length + filteredSystemEntries.length;

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

  const handleClean = async (useTrash: boolean) => {
    setConfirmOpen(false);
    useUiStore.getState().setAppState('cleaning');

    const artifactPaths: string[] = [];
    const systemPathsToDelete: string[] = [];
    const cacheIdsToClean: string[] = [];

    for (const key of selectedPaths) {
      if (systemKeys.has(key)) {
        const cacheId = cacheKeyToId.get(key);
        if (cacheId) {
          cacheIdsToClean.push(cacheId);
        } else {
          systemPathsToDelete.push(key.slice(SYSTEM_PREFIX.length));
        }
      } else {
        artifactPaths.push(key);
      }
    }

    const allPathsToDelete = [...artifactPaths, ...systemPathsToDelete];
    const hasPathClean = allPathsToDelete.length > 0;
    const hasCacheClean = cacheIdsToClean.length > 0;

    if (hasPathClean) {
      useCleanStore.getState().startClean(allPathsToDelete, {
        use_trash: useTrash,
        dry_run: false,
        force: false,
      });
    }

    if (hasCacheClean) {
      try {
        await Promise.all(cacheIdsToClean.map((id) => commands.cleanCache(id)));
      } catch (err) {
        console.error('Cache clean error:', err);
      }
    }

    if (!hasPathClean && hasCacheClean) {
      useUiStore.getState().setAppState('done');
    }

    useSystemStore.getState().detectSystem().catch(console.error);
  };

  if (!result) return null;

  return (
    <div className="flex flex-col h-full">
      {/* Summary bar */}
      <DiskBar
        totalCleanable={totalCleanable}
        selectedSize={selectedSize}
        selectedCount={selectedPaths.size}
        artifactCount={flatArtifacts.length}
        projectCount={result.projects.length}
        systemItemCount={systemEntries.length}
        diskUsed={diskUsed}
        diskTotal={diskTotal}
      />

      {/* Impact cards — top 3 biggest items */}
      <ImpactCards
        items={impactItems}
        selectedPaths={selectedPaths}
        onToggle={togglePath}
      />

      {/* Toolbar with search, presets, view toggle */}
      <ViewToolbar
        allSelected={allSelected}
        someSelected={selectedPaths.size > 0 && !allSelected}
        onSelectAll={selectAll}
        onDeselectAll={deselectAll}
        groupCount={groups.length + systemCategoryCount}
        itemCount={filteredItemCount}
        totalItemCount={totalItemCount}
        presetsSlot={
          <SelectionPresets
            flatArtifacts={flatArtifacts}
            systemEntries={systemEntries}
            onApply={applyPreset}
          />
        }
      />

      {/* List content */}
      <div className="flex-1 overflow-y-auto">
        {viewMode === 'grouped' ? (
          <>
            <GroupedList
              groups={groups}
              selectedArtifacts={selectedPaths}
              collapsedGroups={collapsedGroups}
              maxSize={maxSize}
              onToggleArtifact={togglePath}
              onToggleGroup={toggleGroup}
              onToggleCollapse={toggleCollapse}
            />

            {/* System error banner */}
            {systemError && (
              <div className="flex items-center gap-2 px-5 py-2 bg-[var(--color-danger)]/10 border-b border-[var(--color-border)] text-xs text-[var(--color-danger)]">
                System detection failed: {systemError}
              </div>
            )}

            {(filteredSystemEntries.length > 0 || isDetectingSystem) && (
              <div className="flex items-center gap-2 px-5 py-2.5 bg-[var(--color-bg)] border-y border-[var(--color-border)]">
                <Monitor size={14} className="text-[var(--color-text-muted)]" />
                <span className="text-xs font-semibold uppercase tracking-wider text-[var(--color-text-muted)]">
                  System & Caches
                </span>
                {isDetectingSystem && (
                  <Loader2 size={12} className="animate-spin text-[var(--color-text-muted)]" />
                )}
              </div>
            )}

            <SystemSection
              entries={filteredSystemEntries}
              selectedPaths={selectedPaths}
              maxSize={maxSize}
              onToggle={togglePath}
              onToggleCategory={toggleSystemCategory}
            />
          </>
        ) : (
          <FlatList
            items={filteredArtifacts}
            systemEntries={filteredSystemEntries}
            selectedArtifacts={selectedPaths}
            maxSize={maxSize}
            onToggleArtifact={togglePath}
          />
        )}

        {filteredItemCount === 0 && !isDetectingSystem && (
          <div className="flex items-center justify-center h-full text-sm text-[var(--color-text-muted)]">
            {query ? 'No items match your search' : 'No cleanable items found'}
          </div>
        )}
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
