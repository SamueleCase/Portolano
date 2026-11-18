import { type QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState } from "react";

import { ToastViewport } from "../components/ToastViewport";
import { visibleText } from "../components/visibleText";
import { BackgroundCanvas } from "../design/background/BackgroundCanvas";
import { FirstRun } from "../features/first-run/FirstRun";
import { SessionServicesContext } from "../features/sessions/sessionServices";
import { useSessionEndedEvents } from "../features/sessions/useSessionEndedEvents";
import { texts } from "../i18n/it";
import { presentFailure } from "../ipc/errors";
import type { SessionTransport } from "../ipc/sessions";
import { useToasts } from "../state/toasts";
import styles from "./App.module.css";
import { FeatureHost } from "./FeatureHost";
import { featureRegistry } from "./featureRegistry";
import { type FeatureRegistry, FeatureRegistryContext } from "./features";
import { MainWindow } from "./MainWindow";
import { appSessionTransport } from "./sessionTransport";
import { useAppearance } from "./useAppearance";
import { useStartup } from "./useStartup";
import { useWindowActionsValue, WindowActionsContext } from "./windowActions";
import { createWindowServices, WindowServicesContext } from "./windowServices";

interface AppProps {
  queryClient: QueryClient;
  /** The transport of the sessions: the one of Tauri unless a test gives another. */
  transport?: SessionTransport;
  /** The functions of the hook points: those of the app unless a test gives others. */
  features?: FeatureRegistry;
}

/** The root of the interface: the providers, then the window. */
export function App({
  queryClient,
  transport = appSessionTransport,
  features = featureRegistry,
}: AppProps) {
  const [services] = useState(() =>
    createWindowServices(transport, queryClient, {
      canReadyVault: (dialog) => features[dialog] !== undefined,
    }),
  );
  return (
    <QueryClientProvider client={queryClient}>
      <FeatureRegistryContext value={features}>
        <WindowServicesContext value={services}>
          <SessionServicesContext value={services.sessions}>
            <Shell />
          </SessionServicesContext>
        </WindowServicesContext>
      </FeatureRegistryContext>
    </QueryClientProvider>
  );
}

/**
 * The window over the animated background: loading, the first run without a
 * document, or the main window; the toasts and the functions of the hook
 * points over them, all with the actions of the window.
 */
function Shell() {
  const actions = useWindowActionsValue();
  useAppearance();
  useSessionEndedEvents();
  const startup = useStartup();
  const toasts = useToasts((state) => state.toasts);
  const dismiss = useToasts((state) => state.dismiss);

  return (
    <WindowActionsContext value={actions}>
      <BackgroundCanvas />
      {startup.view === "loading" && (
        <p className={styles.status} role="status" aria-busy="true">
          {texts.app.loading}
        </p>
      )}
      {startup.view === "failed" && (
        <p className={styles.status} role="alert">
          {texts.app.startFailed}: {visibleText(presentFailure(startup.error).message)}
        </p>
      )}
      {startup.view === "firstRun" && <FirstRun />}
      {startup.view === "document" && <MainWindow status={startup.status} />}
      <FeatureHost />
      <ToastViewport toasts={toasts} onDismiss={dismiss} />
    </WindowActionsContext>
  );
}
