/** The texts of the first run and of the empty document (inventario §5.1, §5.12). */
export const firstRun = {
  label: "Primo avvio",
  lead: "Le connessioni vivono in un documento. Creane uno nuovo o apri quello che usi già.",
  create: {
    title: "Crea un nuovo documento",
    text: "Vuoto, pronto per la prima connessione o per un'importazione.",
  },
  open: {
    title: "Apri un documento esistente",
    text: "Un file .json dal disco.",
  },
  recent: "Documenti recenti",
  empty: {
    title: "Il documento è vuoto",
    text: {
      keychain:
        "Aggiungi la prima connessione oppure importa quelle che hai già in Royal TSX. Le password restano nel portachiavi di sistema.",
      file: "Aggiungi la prima connessione oppure importa quelle che hai già in Royal TSX. Le password restano nel vault cifrato.",
    },
    newConnection: "Nuova connessione",
    import: "Importa da Royal TSX",
  },
} as const;
