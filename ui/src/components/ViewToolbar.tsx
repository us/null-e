import { CheckSquare, Square, MinusSquare, LayoutGrid, List } from 'lucide-react';
import { useUiStore } from '@/stores/ui-store';
import type { FlatSortBy } from '@/stores/ui-store';
import { SearchBar } from '@/components/shared/SearchBar';

interface ViewToolbarProps {
  allSelected: boolean;
  someSelected?: boolean;
  onSelectAll: () => void;
  onDeselectAll: () => void;
  groupCount: number;
  itemCount: number;
  totalItemCount?: number;
  presetsSlot?: React.ReactNode;
}

export function ViewToolbar({
  allSelected,
  someSelected = false,
  onSelectAll,
  onDeselectAll,
  groupCount,
  itemCount,
  totalItemCount,
  presetsSlot,
}: ViewToolbarProps) {
  const viewMode = useUiStore((s) => s.viewMode);
  const flatSortBy = useUiStore((s) => s.flatSortBy);
  const searchQuery = useUiStore((s) => s.searchQuery);
  const setViewMode = useUiStore((s) => s.setViewMode);
  const setFlatSortBy = useUiStore((s) => s.setFlatSortBy);
  const setSearchQuery = useUiStore((s) => s.setSearchQuery);

  const isFiltered = searchQuery.trim().length > 0;
  const selectIcon = allSelected
    ? <CheckSquare size={14} />
    : someSelected
      ? <MinusSquare size={14} />
      : <Square size={14} />;

  return (
    <div className="shrink-0 flex items-center gap-3 px-5 py-2 border-b border-[var(--color-border)]">
      {/* Left: Select all / Deselect all */}
      <button
        onClick={allSelected ? onDeselectAll : onSelectAll}
        className="shrink-0 flex items-center gap-1.5 text-xs text-[var(--color-text-secondary)] hover:text-[var(--color-text)] transition-colors focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] outline-none rounded"
        aria-label={allSelected ? 'Deselect all' : 'Select all'}
      >
        {selectIcon}
        {allSelected ? 'Deselect all' : 'Select all'}
      </button>

      {/* Presets slot */}
      {presetsSlot}

      {/* Search */}
      <div className="flex-1 max-w-[220px]">
        <SearchBar
          value={searchQuery}
          onChange={setSearchQuery}
          placeholder="Filter items..."
        />
      </div>

      {/* Filter indicator */}
      {isFiltered && totalItemCount != null && (
        <span className="text-[11px] text-[var(--color-primary)]">
          {itemCount} of {totalItemCount}
        </span>
      )}

      <div className="flex items-center gap-3 ml-auto">
        {/* Sort dropdown (flat mode only) */}
        {viewMode === 'flat' && (
          <select
            value={flatSortBy}
            onChange={(e) => setFlatSortBy(e.target.value as FlatSortBy)}
            className="text-xs bg-[var(--color-bg-secondary)] text-[var(--color-text-secondary)] border border-[var(--color-border)] rounded-md px-2 py-1 cursor-pointer outline-none focus:border-[var(--color-primary)]"
            aria-label="Sort by"
          >
            <option value="size">Size</option>
            <option value="name">Name</option>
            <option value="technology">Technology</option>
          </select>
        )}

        {/* Info text */}
        <span className="text-xs text-[var(--color-text-muted)]">
          {viewMode === 'grouped'
            ? `${groupCount} categories`
            : `${itemCount} items`}
        </span>

        {/* Segmented control */}
        <div className="flex items-center bg-[var(--color-bg-tertiary)] rounded-lg p-0.5">
          <SegmentButton
            active={viewMode === 'grouped'}
            onClick={() => setViewMode('grouped')}
            title="Grouped view"
          >
            <LayoutGrid size={14} />
          </SegmentButton>
          <SegmentButton
            active={viewMode === 'flat'}
            onClick={() => setViewMode('flat')}
            title="Flat view"
          >
            <List size={14} />
          </SegmentButton>
        </div>
      </div>
    </div>
  );
}

function SegmentButton({
  active,
  onClick,
  title,
  children,
}: {
  active: boolean;
  onClick: () => void;
  title: string;
  children: React.ReactNode;
}) {
  return (
    <button
      onClick={onClick}
      title={title}
      aria-label={title}
      className={`flex items-center justify-center w-8 h-8 rounded-md transition-all focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] outline-none ${
        active
          ? 'bg-[var(--color-bg)] text-[var(--color-text)] shadow-sm'
          : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-secondary)]'
      }`}
    >
      {children}
    </button>
  );
}
