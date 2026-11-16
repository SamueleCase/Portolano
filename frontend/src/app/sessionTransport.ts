import type { SessionTransport } from "../ipc/sessions";
import { tauriSessionTransport } from "../ipc/sessionsTauri";

/**
 * The transport of the sessions of the app: the commands and channels of
 * Tauri. The end-to-end tests run the frontend alone, over the fake backend
 * (`main.tsx`), and need the fake transport, where every connection of the
 * fake document is a terminal that works. Vite drops this branch, and the
 * fake with it, from the other builds.
 */
export const appSessionTransport: SessionTransport =
  import.meta.env.MODE === "e2e" ? await fakeTransport() : tauriSessionTransport;

async function fakeTransport(): Promise<SessionTransport> {
  const { FakeSessionTransport, fakeScenarios, workingConnection } =
    await import("../test/fakeSessions");
  return new FakeSessionTransport(fakeScenarios, workingConnection);
}
