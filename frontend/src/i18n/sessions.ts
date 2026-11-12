/** The texts of the session tabs (inventario §5.4, §6; ambiguities 11 and 12). */
export const sessionsTexts = {
  status: {
    connecting: "Connessione in corso",
    connected: "Connesso",
    rdpRunning: "Sessione RDP aperta in una finestra esterna",
    externalOpen: "Aperta in Windows App",
    ended: "Sessione terminata",
    endedWithCode: (code: number) => `Sessione terminata (codice ${code})`,
    failed: "Connessione non riuscita",
    failedWithCode: (code: number) => `Connessione non riuscita (codice ${code})`,
  },
  missingView: "Questo tipo di sessione non si può ancora mostrare.",
} as const;
