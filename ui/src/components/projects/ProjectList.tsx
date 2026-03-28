import { useState, useMemo } from 'react';
import type { ProjectDto } from '@/lib/tauri';
import { SearchBar } from '@/components/shared/SearchBar';
import { ProjectCard } from './ProjectCard';

type SortKey = 'size' | 'name' | 'kind';

interface ProjectListProps {
  projects: ProjectDto[];
  selectedArtifacts: Set<string>;
  onToggleArtifact: (path: string) => void;
  onToggleProject: (project: ProjectDto) => void;
}

export function ProjectList({
  projects,
  selectedArtifacts,
  onToggleArtifact,
  onToggleProject,
}: ProjectListProps) {
  const [search, setSearch] = useState('');
  const [sortBy, setSortBy] = useState<SortKey>('size');

  const filtered = useMemo(() => {
    let list = projects;

    if (search) {
      const q = search.toLowerCase();
      list = list.filter(
        (p) =>
          p.name.toLowerCase().includes(q) ||
          p.kind.toLowerCase().includes(q) ||
          p.root.toLowerCase().includes(q),
      );
    }

    const sorted = [...list];
    switch (sortBy) {
      case 'size':
        sorted.sort((a, b) => b.total_size - a.total_size);
        break;
      case 'name':
        sorted.sort((a, b) => a.name.localeCompare(b.name));
        break;
      case 'kind':
        sorted.sort((a, b) => a.kind.localeCompare(b.kind) || b.total_size - a.total_size);
        break;
    }

    return sorted;
  }, [projects, search, sortBy]);

  const sortButtons: { key: SortKey; label: string }[] = [
    { key: 'size', label: 'Size' },
    { key: 'name', label: 'Name' },
    { key: 'kind', label: 'Kind' },
  ];

  return (
    <div className="space-y-4">
      {/* Controls */}
      <div className="flex items-center gap-3">
        <div className="flex-1 max-w-sm">
          <SearchBar value={search} onChange={setSearch} placeholder="Search projects..." />
        </div>
        <div className="flex items-center gap-1 rounded-lg border border-[var(--color-border)] p-0.5">
          {sortButtons.map((btn) => (
            <button
              key={btn.key}
              onClick={() => setSortBy(btn.key)}
              className={`px-3 py-1 rounded-md text-xs font-medium transition-colors ${
                sortBy === btn.key
                  ? 'bg-[var(--color-primary)] text-white'
                  : 'text-[var(--color-text-secondary)] hover:text-[var(--color-text)]'
              }`}
            >
              {btn.label}
            </button>
          ))}
        </div>
      </div>

      {/* Results count */}
      <p className="text-xs text-[var(--color-text-muted)]">
        {filtered.length} project{filtered.length !== 1 ? 's' : ''}
        {search && ` matching "${search}"`}
      </p>

      {/* List */}
      <div className="space-y-2">
        {filtered.map((project) => {
          const projectArtifactPaths = project.artifacts.map((a) => a.path);
          const allSelected =
            projectArtifactPaths.length > 0 &&
            projectArtifactPaths.every((p) => selectedArtifacts.has(p));

          return (
            <ProjectCard
              key={project.id}
              project={project}
              selectedArtifacts={selectedArtifacts}
              onToggleArtifact={onToggleArtifact}
              onToggleProject={onToggleProject}
              allSelected={allSelected}
            />
          );
        })}
      </div>

      {filtered.length === 0 && (
        <div className="flex items-center justify-center h-32 rounded-xl border border-dashed border-[var(--color-border)] text-[var(--color-text-muted)] text-sm">
          {search ? 'No projects match your search' : 'No projects found'}
        </div>
      )}
    </div>
  );
}
