import { Search, X } from 'lucide-react';

interface SearchBarProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
}

export function SearchBar({ value, onChange, placeholder = 'Search...' }: SearchBarProps) {
  return (
    <div className="relative flex items-center">
      <Search
        size={16}
        className="absolute left-3 text-[var(--color-text-muted)] pointer-events-none"
      />
      <input
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className="w-full pl-9 pr-8 py-2 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] text-sm text-[var(--color-text)] placeholder:text-[var(--color-text-muted)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] focus:border-transparent transition-colors"
      />
      {value && (
        <button
          onClick={() => onChange('')}
          className="absolute right-2 p-0.5 rounded hover:bg-[var(--color-surface-hover)] text-[var(--color-text-muted)]"
        >
          <X size={14} />
        </button>
      )}
    </div>
  );
}
