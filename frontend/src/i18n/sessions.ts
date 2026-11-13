/** The texts of the session tabs (inventario §5.4, §6; ambiguities 11 and 12). */
export const sessions = {
  status: {
    connecting: "Connessione in corso",
    connected: "Connesso",
    rdpRunning: "In corso",
    externalOpen: "Aperta in Windows App",
    endedWithCode: "Sessione terminata (codice {code})",
    failed: "Connessione non riuscita",
    failedWithCode: "Connessione non riuscita (codice {code})",
  },
  actions: {
    retry: "Riprova",
    close: "Chiudi",
    disconnect: "Disconnetti",
    showLog: "Mostra log",
    hideLog: "Nascondi log",
  },
  header: "Stato della sessione",
  rdp: {
    title: "Sessione RDP aperta in una finestra esterna",
    text: "Il desktop remoto è gestito da FreeRDP. Qui vedi solo lo stato della sessione.",
    client: "Client",
    freerdp: "FreeRDP",
    duration: "Durata",
    log: "Log di FreeRDP",
  },
  external: {
    title: "Aperta in Windows App",
    text: "La connessione è gestita da Windows App: chiudere questa tab non la chiude.",
  },
  failure: {
    title: "Connessione non riuscita",
  },
  missingView: "Questo tipo di sessione non si può ancora mostrare.",
  notFound: "La sessione non è più aperta.",
} as const;

/**
 * The texts of the sessions section of the gallery (`?gallery`): imported only
 * by the gallery, so they stay out of the production bundle.
 */
export const sessionsGallery = {
  tabs: "Tab delle sessioni",
  open: {
    ssh: "Apri SSH",
    refused: "Apri SSH rifiutata",
    rdp: "Apri RDP",
    rdpFailed: "Apri RDP non riuscita",
    windowsApp: "Apri Windows App",
  },
  names: {
    "node-ssh-ok": "web01",
    "node-ssh-refused": "web02",
    "node-rdp": "dc01",
    "node-rdp-failed": "dc02",
    "node-windows-app": "ts01",
  },
  summaries: {
    "node-ssh-ok": "operatore@web01.example.com:22",
    "node-ssh-refused": "operatore@web02.example.com:22",
    "node-rdp": "EXAMPLE\\operatore@192.0.2.10:3389",
    "node-rdp-failed": "EXAMPLE\\operatore@192.0.2.12:3389",
    "node-windows-app": "EXAMPLE\\operatore@192.0.2.11:3389",
  },
  empty: "Nessuna sessione aperta.",
} as const;
