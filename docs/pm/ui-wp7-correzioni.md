# UI-WP7 – Correzioni: revisione di sicurezza e revisione del WP (2026-11-18)

Fonti:
- revisione di sicurezza del backend e della webview su `e4cb6a7`;
- revisione indipendente di UI-WP7 su `3d3b4e0`.

Decisioni del PM in DEC-046. Un finding per commit quando possibile, con un test che fallisce prima della correzione.

## Sicurezza – backend
- **S1 (medio) – comando del terminale personalizzato dalla webview.**
  - Problema: `config_set("terminal.command")` più `open_in_terminal` eseguono una riga di shell scelta dalla webview
    (`commands/settings.rs`, `connectors/terminal.rs` `custom_launcher`).
  - **Correzione scelta dall'utente: conferma nativa** (DEC-046.1).
    - In `config_set`, per `terminal.command` con un valore diverso da quello salvato, Rust apre un dialogo nativo
      di conferma con il testo del comando (caratteri di controllo resi visibili). I bottoni sono "Annulla" e "Usa
      questo comando", con testi in italiano accanto agli altri testi del backend.
    - Se l'utente annulla, nulla si scrive e il comando risponde con un errore riconoscibile. Il campo delle
      Preferenze torna al valore salvato senza mostrare un errore rosso.
    - La conferma passa da un'astrazione iniettabile, così i test non aprono finestre. Test:
      - senza conferma `config.toml` resta invariato e `open_in_terminal` usa il comando vecchio;
      - con la conferma il valore si scrive;
      - lo stesso valore non chiede nulla.
    - E2e sul finto backend: annulla, il campo torna al valore salvato. Il finto simula la risposta del dialogo
      con un seme.
    - README: sezione sicurezza.
- **S2 bis**: ammettere anche `http://tauri.localhost` con `cfg(windows)` (DEC-046.9).
- **S3 bis**: il test non scrive più nella cache reale dell'utente; cartella temporanea dei connettori configurabile
  per i test (DEC-046.10).
- **W5 bis**: `picomatch` senza correzione, accettato (DEC-046.8). Nel README, fra i limiti o nei test, una riga con
  l'esito di `npm audit`.
- **S2 (basso) – guardia di navigazione.**
  - Un plugin minimo con `on_navigation` sulla finestra principale ammette solo l'origine dell'app: `tauri://localhost`,
    più `devUrl` solo in sviluppo. Rifiuta tutto il resto, `portolano-icon:` compreso.
  - Test.
- **S3 (basso) – sentinella Rust più ampia** (`src/tests.rs`). Aggiungere:
  - `create_file_vault`;
  - sblocco con master password errata;
  - errori di `set_secret`, `edit_node` e `import_apply` (risposte non allineate) mentre ricevono un segreto;
  - assenza di `TEST_MASTER_PASSWORD` da risposte, eventi, log e file;
  - la directory temporanea della cache (log di FreeRDP, `.rdp`) fra i file letti.
- **S4 – CSP su tutte le risposte dello schema icone** (`icons.rs`): `default-src 'none'; sandbox` per ogni tipo, non
  solo SVG. Test.
- **S5 – password RDP azzerata subito dopo la scrittura su stdin.**
  - Va rilasciata subito, non a fine sessione (`sessions/rdp.rs`).
  - Eccezione minima su `crates/portolano-connectors` per un `take_stdin_secret()` o equivalente. Test.
- **S6 – guard sullo slot della master password** (`commands/vault.rs`): `clear()` nel `Drop`, anche se `contains` va in
  panic. Test.

## Sicurezza – webview
- **W1 (basso) – link del terminale** (`components/terminal/Terminal.tsx`).
  - Si aprono solo con ⌘-clic, come Terminale di macOS.
  - Al passaggio del mouse mostrano l'indirizzo vero con il `Tooltip` dei componenti.
  - Resta il controllo `http(s)`. Test.
- **W2 (basso) – `visibleText` su tutto il testo che può venire dal documento, dal file Royal o dall'host**
  (DEC-032.2):
  - `FormField` (etichetta e messaggio), `GroupError`, `EditorNotices`, `CredentialFacts`;
  - ogni punto che mostra il `message` di un `IpcError` (es. `presentFailure`, toast, banner);
  - `SessionState::Failed.reason` delle sessioni (coda del log di FreeRDP, nella tab e nell'evento `session-ended`).

  Test con caratteri bidi e di controllo.
- **W3 – annulla nei campi della master password.** `UnlockVaultDialog` e `CreateVaultDialog` rimontano i campi dopo
  ogni invio (`key={generation}` come `SecretDialog`). Test.
- **W4 – `MaskedKeyField`.** Blocca anche il trascinamento (`onDragStart`); `writingsuggestions="false"` se i tipi di
  React lo accettano senza `as` non motivati.
- **W5 – `npm audit`** (DEC-046.3): `eslint` 10.15.0 e `typescript-eslint` 8.73.0 (dipendenze di sviluppo).
  - Eccezione su `frontend/package.json` e `package-lock.json`.
  - Dopo l'aggiornamento: lint verde e `npm audit` pulito.

## Revisione di UI-WP7
- **R1 (importante)** `check-bundle.sh`: confrontare l'UUID Mach-O (`dwarfdump --uuid`) del sidecar **nel bundle** con
  quello di `target/release/portolano`. Dimostrare che il controllo fallisce con un binario diverso.
- **R2** `secretsPage.ts`: registrare anche gli argomenti della console (`message.args()` → `jsonValue()`
  serializzato). Sonda con un oggetto.
- **R3** Registro anche del trasporto finto delle sessioni: risposte di `open_session`, messaggi dei canali di stato e
  output.
  - Eccezione su `frontend/src/test/fakeSessions.ts`, solo registrazione.
  - Le credenziali all'avvio vanno cercate anche lì.
- **R4** `secrets.spec.ts`: verificare che ogni segreto arrivi davvero al backend: segreto "presente" nel dettaglio
  dopo Salva e dopo la password dal dettaglio, elemento "Completo" nel report d'import, vault sbloccato e creato.
- **R5** `keyboardPage.ts`:
  - il focus visibile si giudica confrontando lo stile con il focus e dopo `blur()`, non dal colore assoluto;
  - una sola copia dell'helper, importata anche da `gallery-keyboard.spec.ts` (eccezione concessa).
- **R6** `main-window-keyboard.spec.ts`:
  - `expect(stops.length).toBeGreaterThan(1)` in `focusLeaves`;
  - lo sweep confronta le fermate con tutti i controlli attivi della finestra, come la galleria.
- **R7** `secrets.spec.ts:196`: etichetta da `texts`.
- **R8** Copie rimaste dei percorsi leggibili:
  - `features/import/importPath.ts`, `features/import/OptionsStep.tsx` e `features/validation/treePaths.ts` usano
    `components/readablePath.ts`;
  - il separatore passa in una chiave comune di `i18n` (eccezione su questi file e sulle righe di `i18n`).
- **R9** Seme `vaultMissing` del finto: approvato.
- **R10** README:
  - `/Applications`;
  - la frase sulla PTY diventa "l'app non scrive mai nella PTY un segreto salvato (DEC-030.3)";
  - Tab "dovrebbe raggiungere… da verificare dal vivo" (DEC-036.1, DEC-045.1);
  - poi la sezione sicurezza aggiornata con S1, S2 e W1.
- **R11** `i18n/app.ts` `connection.noLoginDialog`: se non si raggiunge più (il dialogo è sempre registrato), togliere
  testo e ramo come `vaultUnavailable` (eccezione su `app/` e `i18n/app.ts`).

## Terzo giro (secondo giro delle revisioni, 2026-11-19; DEC-047)
Fonti: il secondo giro delle tre revisioni su `3708947`, più `30dd20d` (rivisto dal PM: approvato).

### Backend
- **T1 (medio, obbligatoria)** – `dialogs.rs` `confirm_terminal_command`: "Annulla" è il bottone predefinito (Invio).
  - Conferma solo il clic su "Usa questo comando", con `blocking_show_with_result()` o equivalente e un confronto per
    etichetta.
  - Test della funzione che decide quale risultato vale come conferma.
  - Il comportamento di Esc si annota nei test manuali.
- **T2 (basso)**:
  - `portolano_core::text::visible` rende visibili anche Zl, Zp e Cf (U+2028, U+2029, U+200B–U+200D, U+2060, U+FEFF…),
    con test; lo smoke e i test della CLI restano verdi;
  - `confirm_terminal_command` (o `config_set` prima del dialogo) rifiuta un comando con a capo di qualunque tipo o
    più lungo di 1024 caratteri, con un errore di validazione sul campo;
  - una sola conferma aperta alla volta: le altre richieste si rifiutano;
  - test per ognuno.
- **T3 (informativo)**: un test che `run()` registri il plugin della guardia di navigazione, se fattibile senza aprire
  finestre; altrimenti annotato.

### Webview
- **T4 (basso)**: tooltip dei link del terminale (`Terminal.tsx`):
  - host per primo e mai accorciato; percorso accorciato al centro;
  - credenziali `utente@` evidenti;
  - larghezza massima e aggancio ai bordi della finestra;
  - `setHint(undefined)` nella pulizia dell'effetto;
  - test con un URL lungo e con `https://intranet.example.com@evil.example/`.
- **T5**: `visibleText` del frontend allineata a T2 (Zl, Zp, Cf), con test.

### Revisione del WP
- **T6** `check-bundle.sh`: `uuid()` fallisce se `dwarfdump` fallisce o non dà un UUID, così scattano i messaggi
  "manca…" e "non è un binario Mach-O". Quando l'UUID non coincide, il messaggio suggerisce di rieseguire
  `npm run bundle`.
- **T7** README:
  - righe 783-784 rimesse a capo a 120 caratteri, con la frase sul sidecar chiusa prima della firma;
  - riga di `npm audit` indipendente dalla data (DEC-046.8);
  - sicurezza aggiornata con T1 e T2.
- **T8** `secretsPage.ts`: `messageText(message).catch(() => "<argomento non leggibile>")`.
- **T9** `app/sessionTransport.ts`: l'errore `vaultLocked` dell'involucro e2e entra nel registro (eccezione concessa).
- **T10** `keyboardPage.ts`: lo stato "senza focus" si misura prima di raggiungere la fermata, invece di `blur()` e
  `focus()`, se il giudizio resta affidabile; altrimenti un commento che spiega il limite.
- **T11** `features/import/importPath.ts`: `readableImportPath` diventa `readablePath`, se le due differenze (`\`
  finale, `/` iniziale) non toccano i percorsi dell'import; altrimenti un commento con il motivo.
