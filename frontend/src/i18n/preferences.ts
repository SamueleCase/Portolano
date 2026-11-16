/** The texts of the preferences (inventario §5.16, SPEC §9). */
export const preferences = {
  title: "Preferenze",
  close: "Chiudi",
  hint: "Le modifiche si applicano subito.",
  browse: "Sfoglia…",
  reset: "Ripristina",
  groups: {
    document: "Documento",
    vault: "Vault",
    terminal: "Terminale esterno",
    ssh: "Programma SSH",
    rdp: "RDP",
    appearance: "Aspetto",
  },
  document: { label: "Documento predefinito" },
  vault: {
    label: "Dove stanno i segreti",
    backends: { keychain: "Portachiavi di sistema", file: "File cifrato" },
    file: "File del vault",
    fileDefault: "Nella cartella dei dati di Portolano",
  },
  terminal: {
    label: "Terminale per «Apri in terminale esterno»",
    apps: { terminal: "Terminal", iterm: "iTerm", ghostty: "Ghostty", custom: "Comando custom" },
    command: "Comando",
    commandHint: "Scrivi {cmd} dove va il comando di Portolano.",
  },
  ssh: {
    label: "Programma ssh",
    default: "ssh, cercato nel PATH",
    hint: "Un percorso assoluto a un programma non modificabile da altri utenti.",
  },
  rdp: {
    client: "Client",
    clients: { auto: "Automatico", freerdp: "FreeRDP", "windows-app": "Windows App" },
    freerdp: "Percorso di FreeRDP",
    freerdpDefault: "Cercato automaticamente",
    cert: "Certificati del server",
    certs: { tofu: "Fidati al primo uso", ignore: "Ignora", deny: "Rifiuta" },
  },
  appearance: {
    density: "Densità",
    densities: { normal: "Comoda", compact: "Compatta" },
    background: "Sfondo animato",
    backgrounds: { on: "Acceso", reduced: "Ridotto", off: "Spento" },
    reduceMotion: "Riduci movimento",
    reduceMotionHint:
      "Ferma lo sfondo e sostituisce gli spostamenti con dissolvenze. Segue anche l'impostazione di sistema.",
  },
} as const;
