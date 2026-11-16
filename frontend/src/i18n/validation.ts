/** The texts of the validation tab (inventario §5.14, §5.17). */
export const validation = {
  title: { one: "{n} problema nel documento", other: "{n} problemi nel documento" },
  none: "Nessun problema nel documento",
  noneHint: "La validazione non ha trovato errori, avvisi o note.",
  recheck: "Ricontrolla",
  checking: "Controllo in corso…",
  counts: "Problemi per gravità",
  list: "Problemi",
  unknownNode: "(nodo non trovato)",
  separator: " / ",
} as const;
