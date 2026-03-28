import { useState, useEffect, useMemo, useCallback } from 'react';
import { ChevronDown, ChevronRight } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import { formatSize } from '@/lib/format';
import { SystemItemRow, type SystemEntry } from './SystemItemRow';

interface SystemCategory {
  category: string;
  icon: string;
  items: SystemEntry[];
  totalSize: number;
}

interface SystemSectionProps {
  entries: SystemEntry[];
  selectedPaths: Set<string>;
  maxSize: number;
  onToggle: (path: string) => void;
  onToggleCategory: (paths: string[]) => void;
}

export function SystemSection({
  entries,
  selectedPaths,
  maxSize,
  onToggle,
  onToggleCategory,
}: SystemSectionProps) {
  const [collapsedCategories, setCollapsedCategories] = useState<Set<string>>(new Set());
  const [initialized, setInitialized] = useState(false);

  const categories = useMemo<SystemCategory[]>(() => {
    const map = new Map<string, SystemEntry[]>();
    for (const entry of entries) {
      if (!map.has(entry.category)) {
        map.set(entry.category, []);
      }
      map.get(entry.category)!.push(entry);
    }
    const result: SystemCategory[] = [];
    for (const [category, items] of map) {
      items.sort((a, b) => b.size - a.size);
      const totalSize = items.reduce((sum, i) => sum + i.size, 0);
      result.push({
        category,
        icon: items[0].icon,
        items,
        totalSize,
      });
    }
    return result.sort((a, b) => b.totalSize - a.totalSize);
  }, [entries]);

  // Default all categories to collapsed on first load
  useEffect(() => {
    if (categories.length > 0 && !initialized) {
      setCollapsedCategories(new Set(categories.map((c) => c.category)));
      setInitialized(true);
    }
  }, [categories, initialized]);

  const toggleCollapse = useCallback((category: string) => {
    setCollapsedCategories((prev) => {
      const next = new Set(prev);
      if (next.has(category)) next.delete(category);
      else next.add(category);
      return next;
    });
  }, []);

  if (entries.length === 0) return null;

  return (
    <>
      {categories.map((cat) => {
        const isCollapsed = collapsedCategories.has(cat.category);
        const selectedCount = cat.items.filter((i) => selectedPaths.has(i.path)).length;
        const allSelected = selectedCount === cat.items.length;

        return (
          <div key={cat.category}>
            {/* Category header */}
            <div className="sticky top-0 z-10 flex items-center gap-3 px-4 py-2.5 bg-[var(--color-bg-secondary)] border-b border-[var(--color-border)]">
              <input
                type="checkbox"
                checked={allSelected}
                ref={(el) => {
                  if (el) el.indeterminate = selectedCount > 0 && !allSelected;
                }}
                onChange={() => onToggleCategory(cat.items.map((i) => i.path))}
                className="shrink-0 w-4 h-4 rounded cursor-pointer"
                aria-label={`Select all ${cat.category}`}
              />

              <button
                onClick={() => toggleCollapse(cat.category)}
                className="flex items-center gap-2.5 flex-1 min-w-0"
              >
                {isCollapsed ? (
                  <ChevronRight size={14} className="shrink-0 text-[var(--color-text-muted)]" />
                ) : (
                  <ChevronDown size={14} className="shrink-0 text-[var(--color-text-muted)]" />
                )}

                <div className="shrink-0 w-8 h-8 flex items-center justify-center rounded-lg bg-[var(--color-bg-tertiary)] text-base">
                  {cat.icon}
                </div>

                <span className="font-semibold text-sm text-[var(--color-text)]">
                  {cat.category}
                </span>

                <span className="text-xs text-[var(--color-text-muted)]">
                  {cat.items.length} item{cat.items.length !== 1 ? 's' : ''}
                </span>
              </button>

              <span className="tabular-nums font-bold text-sm shrink-0 text-[var(--color-text-secondary)]">
                {formatSize(cat.totalSize)}
              </span>
            </div>

            {/* Category items */}
            <AnimatePresence initial={false}>
              {!isCollapsed && (
                <motion.div
                  initial={{ height: 0, opacity: 0 }}
                  animate={{ height: 'auto', opacity: 1 }}
                  exit={{ height: 0, opacity: 0 }}
                  transition={{ duration: 0.2 }}
                  className="overflow-hidden"
                >
                  {cat.items.map((entry) => (
                    <SystemItemRow
                      key={entry.path}
                      entry={entry}
                      selected={selectedPaths.has(entry.path)}
                      onToggle={onToggle}
                      maxSize={maxSize}
                    />
                  ))}
                </motion.div>
              )}
            </AnimatePresence>
          </div>
        );
      })}
    </>
  );
}
