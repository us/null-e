import { ChevronDown, ChevronRight } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import { TechIcon } from '@/components/shared/TechIcon';
import { formatSize } from '@/lib/format';
import { ArtifactListItem } from './ArtifactListItem';
import type { KindGroup } from './ResultsView';

interface GroupedListProps {
  groups: KindGroup[];
  selectedArtifacts: Set<string>;
  collapsedGroups: Set<string>;
  maxSize: number;
  onToggleArtifact: (path: string) => void;
  onToggleGroup: (group: KindGroup) => void;
  onToggleCollapse: (label: string) => void;
}

export function GroupedList({
  groups,
  selectedArtifacts,
  collapsedGroups,
  maxSize,
  onToggleArtifact,
  onToggleGroup,
  onToggleCollapse,
}: GroupedListProps) {
  return (
    <>
      {groups.map((group) => {
        const isCollapsed = collapsedGroups.has(group.label);
        const groupSelectedCount = group.items.filter((i) =>
          selectedArtifacts.has(i.artifact.path)
        ).length;
        const allGroupSelected = groupSelectedCount === group.items.length;

        return (
          <div key={group.label}>
            {/* Group header */}
            <div className="sticky top-0 z-10 flex items-center gap-3 px-4 py-2.5 bg-[var(--color-bg-secondary)] border-b border-[var(--color-border)]">
              {/* Group checkbox */}
              <input
                type="checkbox"
                checked={allGroupSelected}
                ref={(el) => {
                  if (el) el.indeterminate = groupSelectedCount > 0 && !allGroupSelected;
                }}
                onChange={() => onToggleGroup(group)}
                className="shrink-0 w-4 h-4 rounded cursor-pointer"
                aria-label={`Select all ${group.label}`}
              />

              {/* Collapse toggle + icon + label */}
              <button
                onClick={() => onToggleCollapse(group.label)}
                className="flex items-center gap-2.5 flex-1 min-w-0"
              >
                {isCollapsed ? (
                  <ChevronRight size={14} className="shrink-0 text-[var(--color-text-muted)]" />
                ) : (
                  <ChevronDown size={14} className="shrink-0 text-[var(--color-text-muted)]" />
                )}

                <div
                  className="shrink-0 w-8 h-8 flex items-center justify-center rounded-lg"
                  style={{ backgroundColor: `${group.color}15` }}
                >
                  <TechIcon kind={group.kind} size={20} />
                </div>

                <span className="font-semibold text-sm text-[var(--color-text)]">
                  {group.label}
                </span>

                <span className="text-xs text-[var(--color-text-muted)]">
                  {group.projectCount} project{group.projectCount !== 1 ? 's' : ''} · {group.items.length} artifact{group.items.length !== 1 ? 's' : ''}
                </span>
              </button>

              {/* Group total size */}
              <span
                className="tabular-nums font-bold text-sm shrink-0"
                style={{ color: group.color }}
              >
                {formatSize(group.totalSize)}
              </span>
            </div>

            {/* Group items */}
            <AnimatePresence initial={false}>
              {!isCollapsed && (
                <motion.div
                  initial={{ height: 0, opacity: 0 }}
                  animate={{ height: 'auto', opacity: 1 }}
                  exit={{ height: 0, opacity: 0 }}
                  transition={{ duration: 0.2 }}
                  className="overflow-hidden"
                >
                  {group.items.map(({ artifact, projectName, projectRoot, kind }) => (
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
                  ))}
                </motion.div>
              )}
            </AnimatePresence>
          </div>
        );
      })}

      {groups.length === 0 && (
        <div className="flex items-center justify-center h-full text-sm text-[var(--color-text-muted)]">
          No cleanable artifacts found
        </div>
      )}
    </>
  );
}
