import type { ErrorCode } from "../bindings";

/** Texts of the backend errors, by code (SPEC-UI §4.2): the title of a toast or dialog; the message comes from the backend. */
export const ipc = {
  errorTitles: {
    notFound: "Elemento non trovato",
    ambiguous: "Più elementi corrispondono",
    invalid: "Valore non valido",
    referenced: "Elemento usato da altri nodi",
    vaultLocked: "Vault bloccato",
    vault: "Errore del vault",
    modifiedExternally: "Documento modificato da un altro programma",
    documentMissing: "Documento non trovato",
    noDocument: "Nessun documento aperto",
    connect: "Connessione non riuscita",
    io: "Errore di lettura o scrittura",
    internal: "Errore imprevisto",
  } satisfies Record<ErrorCode, string>,
  unexpected: "Il comando non ha risposto come previsto",
} as const;
