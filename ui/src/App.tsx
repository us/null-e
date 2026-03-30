import { useEffect } from 'react';
import { AppBar } from '@/components/AppBar';
import { WelcomeView } from '@/components/WelcomeView';
import { ScanningView } from '@/components/ScanningView';
import { ResultsView } from '@/components/ResultsView';
import { CelebrationView } from '@/components/CelebrationView';
import { SettingsDrawer } from '@/components/SettingsDrawer';
import { DisclaimerModal } from '@/components/DisclaimerModal';
import { UpdateNotification } from '@/components/UpdateNotification';
import { useUiStore } from '@/stores/ui-store';
import { useScanStore } from '@/stores/scan-store';
import { useTheme } from '@/hooks/useTheme';
import { useScanProgress } from '@/hooks/useScanProgress';
import { useFdaCheck } from '@/hooks/useFdaCheck';

export function App() {
  useTheme();

  const disclaimerAccepted = useUiStore((s) => s.disclaimerAccepted);

  if (!disclaimerAccepted) {
    return <DisclaimerModal />;
  }

  return <AppMain />;
}

function AppMain() {
  useScanProgress();
  useFdaCheck();

  const appState = useUiStore((s) => s.appState);
  const result = useScanStore((s) => s.result);
  const scanError = useScanStore((s) => s.error);

  // Transition to results when scan completes
  useEffect(() => {
    if (result && appState === 'scanning') {
      useUiStore.getState().setAppState('results');
    }
  }, [result, appState]);

  // Transition back to welcome on scan error
  useEffect(() => {
    if (scanError && appState === 'scanning') {
      useUiStore.getState().setAppState('welcome');
    }
  }, [scanError, appState]);

  return (
    <div className="flex flex-col h-full">
      <AppBar />
      <UpdateNotification />
      <main className="flex-1 overflow-hidden">
        {appState === 'welcome' && <WelcomeView />}
        {appState === 'scanning' && <ScanningView />}
        {(appState === 'results' || appState === 'cleaning') && <ResultsView />}
        {appState === 'done' && <CelebrationView />}
      </main>
      <SettingsDrawer />
    </div>
  );
}
