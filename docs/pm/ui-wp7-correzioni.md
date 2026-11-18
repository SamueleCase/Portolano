# UI-WP7 – Correzioni: revisione di sicurezza e revisione del WP (2026-11-18)

Fonti:
- revisione di sicurezza del backend e della webview su `e4cb6a7`;
- revisione indipendente di UI-WP7 su `3d3b4e0`.

Decisioni del PM in DEC-046. Un finding per commit quando possibile, con un test che fallisce prima della correzione.

## Sicurezza – backend
- **S1 (medio) – comando del terminale personalizzato dalla webview.**
  - Problema: `config_set("terminal.command")` più `open_in_terminal` eseguono una riga di shell scelta dalla webview
    (`commands/settings.rs`, `connectors/terminal.rs` `custom_launcher`).
  - **Correzione in sospeso**: aspetta la scelta dell'utente (DEC-046.1). Si fa per ultima.
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
