import { useMemo } from 'react';
import { useUiStore } from '@/stores/ui-store';
import { getKindLabel } from '@/components/shared/TechIcon';
import { ArtifactListItem } from './ArtifactListItem';
import { SystemItemRow, type SystemEntry } from './SystemItemRow';
import type { FlatArtifact } from './ResultsView';

/** Unified item for sorting both artifacts and system entries together */
interface SortableItem {
  type: 'artifact' | 'system';
  path: string;
  name: string;
  size: number;
  techLabel: string;
  artifact?: FlatArtifact;
  systemEntry?: SystemEntry;
}

interface FlatListProps {
  items: FlatArtifact[];
  systemEntries?: SystemEntry[];
  selectedArtifacts: Set<string>;
  maxSize: number;
  onToggleArtifact: (path: string) => void;
}

export function FlatList({
  items,
  systemEntries = [],
  selectedArtifacts,
  maxSize,
  onToggleArtifact,
}: FlatListProps) {
  const flatSortBy = useUiStore((s) => s.flatSortBy);

  const sortedItems = useMemo(() => {
    const all: SortableItem[] = [];

    for (const item of items) {
      all.push({
        type: 'artifact',
        path: item.artifact.path,
        name: item.projectName,
        size: item.artifact.size,
        techLabel: getKindLabel(item.kind),
        artifact: item,
      });
    }

    for (const entry of systemEntries) {
      all.push({
        type: 'system',
        path: entry.path,
        name: entry.name,
        size: entry.size,
        techLabel: entry.category,
        systemEntry: entry,
      });
    }

    switch (flatSortBy) {
      case 'size':
        all.sort((a, b) => b.size - a.size);
        break;
      case 'name':
        all.sort((a, b) => a.name.localeCompare(b.name));
        break;
      case 'technology':
        all.sort((a, b) => {
          const cmp = a.techLabel.localeCompare(b.techLabel);
          if (cmp !== 0) return cmp;
          return b.size - a.size;
        });
        break;
    }

    return all;
  }, [items, systemEntries, flatSortBy]);

  if (sortedItems.length === 0) {
    return null;
  }

  return (
    <>
      {sortedItems.map((item) => {
        if (item.type === 'artifact' && item.artifact) {
          const { artifact, projectName, projectRoot, kind } = item.artifact;
          return (
            <ArtifactListItem
              key={artifact.path}
              artifact={artifact}
              projectName={projectName}
              projectRoot={projectRoot}
              kind={kind}
              selected={selectedArtifacts.has(artifact.path)}
              onToggle={onToggleArtifact}
              maxSize={maxSize}
            />
          );
        }
        if (item.type === 'system' && item.systemEntry) {
          return (
            <SystemItemRow
              key={item.systemEntry.path}
              entry={item.systemEntry}
              selected={selectedArtifacts.has(item.systemEntry.path)}
              onToggle={onToggleArtifact}
              maxSize={maxSize}
            />
          );
        }
        return null;
      })}
    </>
  );
}
