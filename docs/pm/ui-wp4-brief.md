# Brief UI-WP4 – Finestra principale (ondata C)

Sei lo sviluppatore di **UI-WP4** di Portolano (app desktop Tauri v2 + React + TypeScript). Lavori nel tuo worktree
(directory corrente). Le regole comuni sono in `docs/pm/brief-common-ui.md`: leggile per prime e rispettale tutte.

## Commit di partenza
`667a14b` su `main`. Primo passo: `git log --oneline -3`; se il commit manca, `git merge --ff-only main`. Nel worktree
nuovo esegui `npm ci` in `frontend/`.

## Obiettivo
La **cornice dell'app** e la finestra principale: barra, barra laterale con albero, ricerca e filtri, drag & drop,
menu contestuale, area a tab con dettaglio e sessioni, eliminazione con annullamento, duplicazione, piè del vault,
banner del documento, primo avvio, scorciatoie e menu nativo; e i **punti d'aggancio** dove UI-WP5 (editor e dialoghi)
e UI-WP6 (import e validazione) monteranno le loro funzioni (DEC-035). Sostituisce lo `StatusScreen` di UI-WP0.

## Fonti (oltre a quelle del brief comune)
- `docs/SPEC-UI.md` per intero; in particolare §1, §4.3–4.5, §4.9, §5.1, §5.4 (finestra principale, dettaglio,
  "+ Connessione", eliminazione annullabile, documento modificato/mancante, caricamenti ed errori), §5.5, §6, §7.
- `docs/ui/inventario.md` §3, §4 (componenti già pronti), **§5.1, §5.2, §5.3, §5.11, §5.12, §5.15, §5.17**, §6, §8;
  `docs/ui/NOTE.md`; il mock `docs/ui/project/*.html` (riferimento visivo, contenuto = dato).
- `docs/DECISIONS.md`: tutte; in particolare DEC-016, DEC-018, DEC-022, DEC-026.4, DEC-029–DEC-035.
- Il codice integrato: componenti e galleria di UI-WP1 (`frontend/src/components/`, `design/`, `gallery/`),
  IPC e finto backend di UI-WP2 (`frontend/src/ipc/`, `frontend/src/test/fakeBackend*.ts`, comandi in
  `crates/portolano-app/src/commands/`), sessioni di UI-WP3 (`frontend/src/features/sessions/`: leggi il commento di
  `SessionServices` con il contratto per l'host: `SessionServicesContext`, `tauriSessionTransport`, `openUrl`,
  `useSessionEndedEvents`, `onRetry`, `onReplaced`, `onActivity`, `route`).

## File di tua proprietà
- `frontend/src/app/**`, `frontend/src/state/**`, `frontend/src/features/{tree,details,first-run}/**`,
  `frontend/src/i18n/{app,tree,details,firstRun}.ts` (o i nomi d'area che scegli) e le righe in `i18n/it.ts`,
  `frontend/e2e/{main-window,tree,details,first-run,delete}*.spec.ts`;
- in `crates/portolano-app/` solo la configurazione della finestra per il tasto Tab (punto 9), con test se possibile.

Non modificare componenti, design, IPC, finto backend e sessioni: se ti serve un cambiamento lì, chiedilo al PM (le
piccole aggiunte al finto backend necessarie ai tuoi e2e si chiedono, non si fanno).

## Compiti
1. **Cornice e stato** (`app/`, `state/`): provider (QueryClient di `ipc/`, servizi delle sessioni), Zustand per
   selezione, tab aperte (dettaglio, sessioni, validazione), pannelli, toast, preferenze dell'interfaccia (densità,
   sfondo, riduci movimento da `ui_prefs_get`), ascolto degli eventi del backend (`useBackendEvents`), scorciatoie.
   Lo sfondo animato di UI-WP1 pilotato davvero: dialogo aperto, attività di sessione (`onActivity`), preferenze.
2. **Punti d'aggancio** (DEC-035): un insieme tipizzato di richieste (`openEditor({mode:'add', kind, parent}|{mode:'edit',
   id})`, `requestPassword(node, field)`, `unlockVault()`, `createFileVault()`, `requestLogin(node) → login?`,
   `openImport()`, `openValidation()`, `openPreferences()`) e un registro in cui una funzione dichiara il proprio
   componente o gestore. Barra, menu contestuale, menu nativo (`menu-action`), scorciatoie e dettaglio emettono le
   richieste; senza registrazione il comando corrispondente è disattivato (con motivo nel tooltip), nessun segnaposto.
   Documenta il contratto in un commento del modulo: UI-WP5 e UI-WP6 lo useranno in fase 2.
3. **Barra** (inventario §5.1): logo, "portolano", nome del documento (mono, ellissi), "+ Connessione" con menu SSH/RDP
   (ambiguità 10), "+ Cartella", "+ Credenziale", "Importa", "Valida", "Preferenze", "Connetti" (disattivato se la
   selezione non è una connessione); `data-tauri-drag-region`; semafori nativi (non i finti del mock, §8).
4. **Barra laterale**: ricerca (nome, host, tag) e filtri (Tutti, SSH, RDP, Credenziali, ★) con il comando `search`;
   albero con Headless Tree e il `TreeRow` di UI-WP1 (`role=tree`, tastiera completa di §5.5: frecce, Home/End,
   Invio = dettaglio, ⌘↩ = connetti, typeahead), conteggio dei discendenti sulle cartelle chiuse, colore come pallino,
   stati vuoti; **drag & drop** per spostare e riordinare con `move_node` (con alternativa da tastiera, es. "Sposta in…"
   dal menu contestuale con il `NodePicker`); menu contestuale per tipo (inventario §4.15, §5.17: cartelle e
   credenziali); doppio clic connette (sessione esistente → in primo piano; fallita → riprova); piè del vault
   (ambiguità 20: "Portachiavi di sistema" senza stato; "Vault su file · bloccato/sbloccato" cliccabile → sblocco).
5. **Area a tab**: barra delle tab di UI-WP1 (⌘1…⌘9, ⌘W), tab dettaglio, tab di sessione montate con il registro
   `sessionViews` di UI-WP3 e i suoi servizi (connessione: `login_request` → se serve, `requestLogin` → `openSession`;
   "Riprova" con `onRetry`/`onReplaced`, la tab sostituita resta selezionata; `route` dalle tappe del piano o dei jump;
   "Apri in terminale esterno" → `open_in_terminal`), tab di validazione come slot di UI-WP6.
6. **Dettaglio** (inventario §5.2, §5.3): riepilogo, credenziale effettiva con la catena (`resolve_trace`, stati di
   ambiguità 23), segreti (stato; "Imposta"/"Cancella" emettono `requestPassword`/`clear_secret`), utilizzi,
   "Mostra comando" (`connect_preview`, con `<vault:…>`), note, proprietà, sorgente d'import; nodo non supportato in
   sola lettura (niente "Modifica"); azioni Connetti, Modifica (`openEditor`), Duplica, Elimina.
7. **Eliminazione e duplicazione**: `preview_delete` → conferma (inventario §5.11: riferimenti diretti + "la ricevono
   per ereditarietà N nodi", contenuto e segreti di una cartella, "Elimina comunque", Invio = Annulla) → `delete_node`
   → toast "Eliminato · Annulla" 10 s (`undo_delete`; `kept` mostrati se non vuoti); `duplicate_node` con selezione
   della copia e avviso delle rinomine (`renamed`).
8. **Documento**: primo avvio (inventario §5.12: nuovo documento, apri, recenti; niente "cartella condivisa"; dialoghi
   nativi via `choose_document`/`choose_new_document`), stato vuoto (frase sul vault secondo il backend), banner
   "modificato all'esterno" con "Ricarica" e "mancante" con "Ricrea / Apri altro" (§5.4), skeleton di caricamento,
   spinner sui bottoni, toast d'errore persistenti (ambiguità 17), mappa degli errori di `ipc/errors.ts`
   (candidati di `ambiguous`, ecc.).
9. **Tasto Tab in WKWebView** (DEC-032.4): con la sola configurazione della finestra (preferenza di WKWebView
   impostata da Rust, es. `tabFocusesLinks` o l'equivalente che fa raggiungere bottoni e controlli con Tab; verifica
   cosa espone `tauri`/`wry` 2.12) l'app deve raggiungere tutti i controlli con Tab anche senza la navigazione da
   tastiera di macOS. Se non è possibile senza codice non sicuro o API private, fermati e riporta le opzioni.
10. **Menu nativo**: gestisci tutti i `menu-action` di UI-WP2 (§4.9) con le stesse richieste di barra e scorciatoie.
11. **e2e** sul finto backend (WebKit, CSP e prototipo congelato già nelle fixture): primo avvio → nuovo documento;
    albero, ricerca, filtri, tastiera; drag & drop e riordino; dettaglio con catena e "Mostra comando"; eliminazione
    con annullamento (e scadenza); duplicazione; banner di modifica esterna; connessione SSH con tab e sessione finta;
    screenshot di ogni schermata in `frontend/e2e-screenshots/main-window/` per il checkpoint visivo 2.

## Gate prima di consegnare (output reale nel report)
Rust: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
In `frontend/`: `npm run typecheck`, `npm run lint`, `npm run format:check`, `npm test`, `npm run build`,
`npm run bindings:check`, `npm run e2e` (2 volte). Nel report: il contratto dei punti d'aggancio, i percorsi degli
screenshot, l'esito del punto 9.
