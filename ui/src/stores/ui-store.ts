import { create } from 'zustand';

export type AppState = 'welcome' | 'scanning' | 'results' | 'cleaning' | 'done';
export type Theme = 'dark' | 'light' | 'system';
export type ViewMode = 'grouped' | 'flat';
export type FlatSortBy = 'size' | 'name' | 'technology';

interface UiState {
  appState: AppState;
  theme: Theme;
  settingsOpen: boolean;
  viewMode: ViewMode;
  flatSortBy: FlatSortBy;
  searchQuery: string;
  disclaimerAccepted: boolean;

  setAppState: (state: AppState) => void;
  toggleTheme: () => void;
  setTheme: (theme: Theme) => void;
  setSettingsOpen: (open: boolean) => void;
  setViewMode: (mode: ViewMode) => void;
  setFlatSortBy: (sortBy: FlatSortBy) => void;
  setSearchQuery: (query: string) => void;
  acceptDisclaimer: () => void;
}

export const useUiStore = create<UiState>((set) => ({
  appState: 'welcome',
  theme: 'dark',
  settingsOpen: false,
  viewMode: 'grouped',
  flatSortBy: 'size',
  searchQuery: '',
  disclaimerAccepted: (() => {
    try { return localStorage.getItem('null-e:disclaimer-accepted') !== null; }
    catch { return false; }
  })(),

  setAppState: (appState) => set({ appState }),

  toggleTheme: () =>
    set((state) => {
      const cycle: Theme[] = ['dark', 'light', 'system'];
      const idx = cycle.indexOf(state.theme);
      return { theme: cycle[(idx + 1) % cycle.length] };
    }),

  setTheme: (theme) => set({ theme }),

  setSettingsOpen: (settingsOpen) => set({ settingsOpen }),

  setViewMode: (viewMode) => set({ viewMode }),

  setFlatSortBy: (flatSortBy) => set({ flatSortBy }),

  setSearchQuery: (searchQuery) => set({ searchQuery }),

  acceptDisclaimer: () => {
    localStorage.setItem('null-e:disclaimer-accepted', new Date().toISOString());
    set({ disclaimerAccepted: true });
  },
}));
