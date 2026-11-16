# Brief UI-WP5 – Editor e dialoghi (ondata C)

Sei lo sviluppatore di **UI-WP5** di Portolano (app desktop Tauri v2 + React + TypeScript). Lavori nel tuo worktree
(directory corrente). Le regole comuni sono in `docs/pm/brief-common-ui.md`: leggile per prime e rispettale tutte.

## Commit di partenza
`667a14b` su `main`. Primo passo: `git log --oneline -3`; se il commit manca, `git merge --ff-only main`. Nel worktree
nuovo esegui `npm ci` in `frontend/`.

## Due fasi (DEC-035)
- **Fase 1 (ora)**: le funzioni, autonome, con i componenti di UI-WP1 e l'IPC di UI-WP2: ogni funzione espone un
  componente o un gestore con props e callback chiari (chi lo apre, con quali dati, cosa restituisce). Test Vitest sul
  finto backend, una sezione della galleria o pagine e2e. Consegna il report di fase 1 e fermati.
- **Fase 2 (dopo il merge di UI-WP4; il PM ti riprende)**: `git merge main` e registrazione delle funzioni nei punti
  d'aggancio della cornice di UI-WP4 (righe d'integrazione in `frontend/src/app/**` e `frontend/src/state/**`
  concesse), e2e dei flussi completi nella finestra principale, screenshot per il checkpoint visivo 2.

## Fonti (oltre a quelle del brief comune)
- `docs/SPEC-UI.md` per intero; in particolare §1 (soprattutto 1.2: segreti in ingresso), §4.3, §4.4 (osservazione
  del file e patch dell'editor), §4.6, §5.4 (editor, "+ Connessione", dialoghi, preferenze), §5.5, §6, §7.
- `docs/ui/inventario.md` §4 (componenti), **§5.5, §5.6, §5.7, §5.8, §5.9, §5.10, §5.16, §5.17**, §6, §7, §8;
  `docs/ui/NOTE.md`; il mock `docs/ui/project/*.html` (riferimento visivo, contenuto = dato).
- `docs/DECISIONS.md`: tutte; in particolare DEC-016.3–4, DEC-022, DEC-024, DEC-026.1, DEC-029, DEC-031 (staging,
  `keyFile`, `choose_key_file`), DEC-032, DEC-033.8, DEC-035.
- Il codice integrato: componenti di UI-WP1 (`SidePanel`, `Stepper`, `CredentialField`, `CredentialSelector`,
  `CredentialPreview`, `IconPicker`, `ColorPicker`, `NodePicker`, `SortableList`, `Dialog` con `onSubmit`,
  `SecretBadge`, `TagInput`, `KeyValueGrid`, …), IPC e finto backend di UI-WP2 (`useSecretMutation`, comandi
  `add_node`, `edit_node` con `EditorChanges`, `set_secret`, `clear_secret`, `unlock_vault`, `create_file_vault`,
  `login_request`, `config_*`, `ui_prefs_*`, `choose_*`, `custom_icons`, `builtin_icons`, `add_icon`).

## File di tua proprietà
- `frontend/src/features/{editor,dialogs,preferences}/**`, `frontend/src/i18n/{editor,dialogs,preferences}.ts` e le
  righe in `i18n/it.ts`, una sezione nuova in `frontend/src/gallery/sections/` con la riga che la registra,
  `frontend/e2e/{editor,dialogs,preferences}*.spec.ts`;
- in fase 2 le righe d'integrazione concesse in `app/` e `state/`.
Non modificare componenti, design, IPC e finto backend: se ti serve un cambiamento lì, chiedilo al PM.

## Compiti – fase 1
1. **Editor in due passi** (inventario §5.5, SPEC-UI §5.4): pannello laterale; passo 1 Generale (nome, posizione con
   "Cambia…" e `NodePicker` filtrato sulle cartelle, descrizione, note, icona, colore, preferito, tag, proprietà);
   passo 2 Connessione/Accesso/Credenziale predefinita secondo il tipo (SSH: host, porta, utente, jump host con
   `SortableList` e "+ Dall'albero…"/"+ Indirizzo libero", opzioni ssh ammesse DEC-026.1 con l'errore del backend sul
   campo, credenziale con `CredentialSelector`; RDP: host, porta, schermo, gateway, client; cartella: credenziale
   predefinita; credenziale: utente, dominio, password, chiave privata o file con "Sfoglia…" → `choose_key_file`,
   passphrase). **Salva disponibile in entrambi i passi**; porta lasciata al predefinito = nessuna porta; nessun
   valore d'esempio precompilato; "+ Connessione" sceglie SSH/RDP e la cartella di destinazione (ambiguità 10, la
   destinazione arriva dall'host ed è modificabile al passo 1). react-hook-form + zod per la forma (solo validazione
   di forma: le regole di dominio sono del backend, i suoi errori `invalid` vanno sul campo indicato da `details`).
2. **Segreti in staging** (SPEC-UI §1.2, ambiguità 14): password, passphrase e chiave digitate nell'editor restano
   solo nel campo finché l'utente non le conferma col dialogo "Imposta", poi badge "Da salvare" e invio **solo** al
   Salva in `edit_node`/`add_node` con `EditorChanges`, tramite `useSecretMutation` (mai in stato React oltre il
   campo, store, cache, log); "Cancella" è una modifica applicata al Salva; vault bloccato → badge "Vault bloccato" e
   richiesta di sblocco al Salva. Un test con segreto sentinella dimostra che dopo il Salva (riuscito o fallito) il
   valore non resta in store, cache di Query o DOM.
3. **Modifiche esterne durante l'editor** (§4.4): se arriva `document-modified-externally` con l'editor aperto, le
   modifiche restano come patch; dopo "Ricarica" si riapplicano e si avvisa per i campi cambiati da entrambe le parti.
4. **Dialoghi** come componenti esportati: password (titolo con campo e nodo, bottone "Imposta"; inventario §5.9),
   sblocco del vault (master password, errore di password errata; §5.10), creazione del vault su file (minimo 12
   caratteri con il testo di DEC-026.6), **credenziali all'avvio** per "Chiedi all'avvio" (DEC-016.3: utente,
   dominio se il protocollo lo chiede, password; restituisce il login al chiamante senza conservarlo; lo useranno
   la connessione di UI-WP4 e "Riprova" delle sessioni, DEC-033.8), selettore di credenziale con dati reali (6
   modalità, "Per nome" cerca in tutto il documento, stati "nessun risultato"/"ambiguo"), selettore d'icona (builtin,
   importate da `custom_icons`, "Immagine" → `choose_icon_file` → `add_icon`), picker di nodi.
5. **Preferenze** (inventario §5.16, SPEC-UI §5.4): documento predefinito, vault (portachiavi / file con percorso dal
   dialogo nativo `choose_config_file`), terminale esterno (Terminal, iTerm, Ghostty, comando con `{cmd}`), programma
   SSH, RDP (client, percorso FreeRDP, certificati), aspetto (densità, sfondo animato acceso/ridotto/spento, riduci
   movimento: `ui_prefs_set`). Le voci di config passano da `config_set`/`config_reset`; errori sul campo.
6. Galleria/e2e di fase 1 sul finto backend: editor (nuova SSH, RDP, cartella, credenziale; modifica con staging),
   dialoghi, preferenze; screenshot in `frontend/e2e-screenshots/editor/`.

## Gate prima di consegnare (output reale nel report, a fine di ogni fase)
In `frontend/`: `npm run typecheck`, `npm run lint`, `npm run format:check`, `npm test`, `npm run build`,
`npm run e2e`; Rust `cargo fmt --all --check`. Nel report di fase 1: l'API esportata di ogni funzione (props,
callback, risultato), che servirà al PM e a UI-WP4.
