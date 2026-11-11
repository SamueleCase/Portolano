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

## Dalla tavola App aggiornata (11/11)
Nessuna delle correzioni sopra risultava ancora applicata. In più:
- **Eliminazione** (dialogo generico): un nodo usato da altri (credenziale, jump host come `bastion`, o una
  cartella che li contiene) mostra l'elenco degli utilizzi e "Elimina comunque"; le credenziali eliminate
  lasciano i nodi in "Chiedi all'avvio", i jump host eliminati vengono segnalati dalla validazione (DEC-018.1).
  Per una cartella mostrare quanti elementi e segreti verranno cancellati.
- **Editor in due passi**: "Salva" disponibile anche al passo 1 (per cambiare solo il nome).
- **"+ Connessione"**: far scegliere il protocollo (SSH o RDP) e la cartella di destinazione; una porta lasciata
  al valore predefinito si salva come "predefinita", non come numero.
- **Dialogo password**: il titolo indica il campo e il nodo (password, passphrase, proprietà "Token API"…), non
  un testo fisso.
- **Validazione**: ogni riga porta al nodo, compresa quella d'errore; il numero di problemi non è fisso.
- **Segreti nell'editor**: "Imposta", "Sostituisci" e "Cancella" non scrivono subito nel portachiavi; si
  applicano al Salva e si annullano con Annulla (vedi `progress.md`).
- Togliere l'apertura del primo avvio dal logo e i valori d'esempio (jump host, gateway, opzioni) nell'editor di un
  nodo nuovo.
