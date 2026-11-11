# Note sul mock della UI

Correzioni da fare al mock in `project/` perché sia coerente con la SPEC e con le decisioni (`../DECISIONS.md`).
Riguardano solo testi e presentazione: il codice del core segue già queste regole.

## Comportamento descritto in modo diverso dalla SPEC
- **Credenziale "Per nome"** (Editor): non risale le cartelle; cerca in tutto il documento e dà errore se il nome è
  ambiguo (SPEC §4.3).
- **Eliminazione di una credenziale usata** (Dialoghi): i nodi che la usavano passano a "Chiedi all'avvio", non
  "senza credenziale" (DEC-016.5).
- **Report d'import** (Dialoghi): una password saltata rende l'elemento *Parziale*; *Saltati* sono solo gli
  elementi del cestino Royal (SPEC §8.3).
- **"Chiedi all'avvio"** (Editor): niente campo "Utente proposto" per ora (DEC-016.3).
- **"Da una cartella condivisa"** (Dialoghi, primo avvio): la condivisione di documenti è fuori scope (SPEC §14);
  togliere la frase.

## Dettagli di presentazione
- **"Mostra comando"** (App, Preferenze): usare il formato reale dei piani (ordine degli argomenti di SSH come §7.2,
  binario FreeRDP `sdl-freerdp`, `/cert:` e lettura delle credenziali da stdin).
- **Terminale personalizzato** (Preferenze): il segnaposto è `{cmd}`, non `{comando}`.
- **Nodo non supportato** (App): nascondere "Modifica" (resta solo "Elimina"), coerente con "in sola lettura".
- **"Dominio" nell'editor RDP**: appartiene alla credenziale inline; con una credenziale ereditata o collegata non
  c'è un dominio separato della connessione.
- **Avanzamento "salto 1 di 2" e latenza "18 ms"**: OpenSSH con `-J` non espone questi dati; usare uno stato unico
  (connessione in corso / connesso / errore).
- **Keepalive**: campo vuoto = disattivato (non "0").
- **Sblocco del vault**: il testo "portachiavi di sistema" va mostrato solo con il backend Keychain; con il vault
  su file indicare il file cifrato. Con il vault bloccato usare il badge "Vault bloccato", non "Assente".
- **Import**: i formati accettati sono `.rtsz`, `.rtsx` e XML (anche compressi gzip o zip).
- **Livelli della validazione**: Errore, Avviso, Nota (DEC-016.6).
