/** The texts of the interface, in Italian (SPEC-UI: no text outside i18n/). */
export const texts = {
  status: {
    title: "Portolano",
    loading: "Caricamento…",
    defaultDocument: "Documento predefinito",
    documentPresent: "(presente)",
    documentAbsent: "(non ancora creato)",
    vault: "Vault",
    vaultKinds: {
      keychain: "Portachiavi di sistema",
      file: "Vault su file",
      memory: "In memoria (solo test)",
    },
    failed: "Impossibile leggere lo stato dell'app",
  },
} as const;
