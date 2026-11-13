# Portolano – Specifica dell'app desktop (FASE 2)

> **Documento-contratto** per l'app con interfaccia grafica. Si legge insieme a `SPEC.md` (core, connettori,
> importer, CLI: invariati salvo dove indicato), `DECISIONS.md`, e alle fonti della UI:
> - `docs/ui/inventario.md`: **fonte dell'aspetto** (token esatti, catalogo dei componenti, schermate, testi);
> - `docs/ui/project/` (mock, solo consultazione) e `docs/ui/NOTE.md` (correzioni al mock);
> - `docs/ui/ricerca-tauri.md`: ricognizione tecnica con versioni verificate e motivazioni.
>
> Se questa specifica contraddice le fonti della UI, vale questa. Le deviazioni vanno in `DECISIONS.md`.
> Lingua: testi dell'interfaccia in **italiano**; codice, identificatori e commenti in **inglese**.

---

## 0. Obiettivo e decisioni

Un'app desktop macOS (Apple Silicon, macOS 14+), **Portolano.app**, che offre in grafica tutto ciò che oggi fa la CLI,
con l'aspetto del mock: albero dei nodi, dettaglio, editor, dialoghi, wizard d'import, validazione, preferenze, e
**sessioni in tab**: terminale SSH integrato e stato delle sessioni RDP (FreeRDP in finestra propria). La CLI resta e
convive con l'app sullo stesso documento, la stessa config e lo stesso vault.

Decisioni che vincolano questa fase: DEC-013 (RDP integrato dopo, con IronRDP), DEC-016 (lock per operazione, lancio in
due passi, chiedi all'avvio, `SecretRef` univoci), DEC-018 e DEC-022 (`portolano-ops` come API dell'app), DEC-024 e
DEC-026 (modello di fiducia, caratteri di controllo), **DEC-027** (Tauri v2 + React + TypeScript, mock definitivo,
spazio per il pannello RDP).

---

## 1. Regole non negoziabili (si aggiungono a SPEC §1)

1. **Il frontend non riceve mai segreti.** Riceve solo lo stato (presente / assente / da salvare / vault bloccato).
   Nessun comando restituisce password, passphrase, chiavi o master password, nemmeno mascherate.
2. **Segreti in ingresso dalla webview**: le password digitate in un `<input type="password">` vanno al backend con
   **un solo** comando, che le avvolge subito in `Secret`. Nel frontend non finiscono mai in store (Zustand, TanStack
   Query), log, `console`, URL, `localStorage`, né in stato React oltre il campo stesso; il campo si svuota appena
   inviato. È un limite noto delle UI web (le stringhe JS non si azzerano): va documentato nel README.
3. **Stringhe non fidate**: nomi, host, utenti, note e testi del documento arrivano da fonti non fidate (DEC-024). Si
   mostrano solo come testo (mai `dangerouslySetInnerHTML`), con i caratteri di controllo, DEL, C1 e bidi resi visibili
   con un unico helper equivalente a quello della CLI (DEC-026.4).
4. **Icone personalizzate** solo come `<img>` servite dallo schema `portolano-icon://` (§4.7), mai SVG inline,
   `<object>` o `<iframe>`.
5. **Nessun contenuto remoto**: font e librerie inclusi nel bundle; CSP stretta (§4.8); niente plugin `shell`, `fs`,
   `http`; comandi permessi elencati esplicitamente (AppManifest); i percorsi dei file li sceglie Rust con i dialoghi
   nativi (§4.6), il frontend non passa percorsi arbitrari.
6. **Accessibilità**: contrasto WCAG AA (testo ≥ 4.5:1, contorni dei controlli ≥ 3:1), tutto raggiungibile da
   tastiera con focus visibile, ruoli e attributi ARIA corretti, `prefers-reduced-motion` rispettato.
7. **Standard di codice**: SPEC §11.3 vale per il Rust dell'app; per il TypeScript valgono gli stessi principi (moduli
   piccoli e coesi, niente codice morto o duplicato, niente `any` né `@ts-ignore` non motivati, nomi del dominio,
   commenti sul perché, test con nome che descrive il comportamento). Dettagli in §7.

---

## 2. Architettura

```
Portolano/
├── Cargo.toml                    # workspace (members = crates/*)
├── crates/
│   ├── portolano-core | -connectors | -import | -ops | -cli   # esistenti
│   └── portolano-app/            # NUOVO: app Tauri (ex "src-tauri")
│       ├── Cargo.toml  build.rs  tauri.conf.json  tauri.bundle.json  capabilities/main.json  icons/
│       └── src/ main.rs lib.rs state.rs error.rs dto/ commands/ sessions/ watch.rs icons.rs menu.rs
└── frontend/                     # NUOVO: Vite + React + TypeScript
    ├── package.json  .nvmrc  vite.config.ts  tsconfig.json  eslint.config.js  .prettierrc  playwright.config.ts
    └── src/ main.tsx app/ bindings.ts (generato) ipc/ state/ design/ components/ features/ i18n/ test/
```

- **Rust (`portolano-app`)**: stato dell'app, comandi IPC sopra `portolano-ops`, sessioni (PTY e processi),
  osservazione del file, schema delle icone, menu nativo. Nessuna logica di dominio duplicata: ciò che manca va
  aggiunto a `portolano-ops` (o ai crate sottostanti) con test, non nell'app.
- **TypeScript (`frontend/`)**: solo presentazione e stato dell'interfaccia. I tipi dei comandi e degli eventi sono
  **generati** da Rust con `tauri-specta` (`bindings.ts`, mai modificato a mano).
- **CLI come sidecar**: il binario `portolano` è incluso nell'app (`Contents/MacOS/portolano`) ed è il programma
  askpass e quello della "nuova finestra" (§4.5).

### 2.1 Versioni (fissate; aggiornamenti solo con una decisione)
Da `docs/ui/ricerca-tauri.md`. Rust: `tauri` 2.12.x, `tauri-build` 2.7.x, `tauri-plugin-dialog` 2.8.x,
`tauri-plugin-opener` 2.7.x, `tauri-specta` e `specta` **`=2.0.0-rc.28`**, `specta-typescript` 0.0.15,
`portable-pty` 0.9.0, `notify` 8.2.x + `notify-debouncer-full` 0.7.x. **Mai** versioni 3.0-alpha.
npm: `@tauri-apps/api` e `@tauri-apps/cli` 2.12.x, React 19.3.x, Vite 8.3.x, `@vitejs/plugin-react` 6.3.x,
**TypeScript `~6.0.3`** (non la 7: typescript-eslint), TanStack Query 5.x, Zustand 5.x, `@headless-tree/*` 1.8.x,
`@tanstack/react-virtual` 3.x, react-hook-form 7.x + zod 4.x + `@hookform/resolvers`, `@xterm/xterm` 6.0.0 +
`addon-fit` 0.11 + `addon-web-links` 0.12, `@fontsource/hanken-grotesk` e `@fontsource/ibm-plex-mono` 5.3.x,
`lucide-react` (set di icone builtin, licenza ISC), ESLint 10 + typescript-eslint 8.72 + react-hooks + react-refresh,
Prettier 3, Vitest 5.1 + jsdom 31 + Testing Library, `@playwright/test` 1.65 (solo WebKit).
**Node 22.23.2** (già installato con nvm; `.nvmrc` = `22.23.2`): la v20 predefinita non basta.

---

## 3. Modifiche ai crate esistenti (UI-WP0)

Piccole, con test, nei file dei rispettivi crate:
1. `LazyVault::lock(&self)`: chiude il vault aperto (il prossimo uso richiede di nuovo la master password).
2. `Store::changed_on_disk(&self) -> Result<bool>`: rilegge sotto lock condiviso e confronta l'impronta ricordata.
3. `portolano_connectors::CHILD_ENV_REMOVE`: costante pubblica con le variabili tolte ai figli (usata da
   `PreparedLaunch::command` e dalla PTY dell'app), invece della lista privata.
4. `OpsError::category() -> ErrorCategory` (in ops), usata sia dalla CLI per i codici d'uscita sia dall'app per
   `IpcError` (la CLI smette di classificare da sé).
5. Feature `specta` facoltativa in core, ops e import: `#[cfg_attr(feature = "specta", derive(specta::Type))]` sui
   tipi **di risultato** che l'app passa così come sono (es. `Issue`, `Severity`, `ImportReport`, `CredentialView`,
   `DeletePreview`, `Entry`). Gli input restano DTO nell'app. Nessun tipo con segreti riceve il derive.
6. Funzioni nuove di ops richieste dall'interfaccia (vedi §5), con l'API in due fasi dove serve una conferma:
   - `resolve_trace(doc, id)`: la catena da cui arriva la credenziale effettiva (nodo → cartelle → credenziale);
   - `credential_usage(doc, id)`: riferimenti diretti (DEC-018.1) **e** nodi che la ricevono per ereditarietà;
   - `duplicate_node(...)`: copia con id nuovi e `SecretRef` nuovi, copiando i valori nel vault (DEC-016.4);
   - eliminazione annullabile: `delete` con l'opzione di **rimandare** la cancellazione dei segreti, che restituisce
     il sottoalbero rimosso e i `SecretRef` da cancellare; `restore_deleted(...)` lo reinserisce;
     `purge_secrets(refs)` li cancella alla scadenza dell'annullamento;
   - `import_requests(...)`: elenco delle richieste di segreti di un import (prima fase, in prova, senza scrivere);
   - `IconStore::list()` per il selettore delle icone personalizzate.
   - **Segreti in staging** dell'editor: `edit_node`/`add_node` ricevono i segreti nuovi o sostituiti insieme alle
     modifiche e li applicano nel percorso di salvataggio unico (segreti → icone → documento, rollback); una
     sostituzione usa un `SecretRef` nuovo e il vecchio si cancella dopo il salvataggio; "Cancella" è una modifica
     dell'editor che si applica al Salva.

---

## 4. Backend dell'app (`portolano-app`)

### 4.1 Stato
```rust
pub struct AppState {
    open: Mutex<Option<OpenDocument>>,          // store + document: load → modifica → save serializzati
    vault: LazyVault,                            // backend dalla config; master password dallo UnlockSlot
    unlock: Arc<UnlockSlot>,                     // MasterPasswordSource: vuoto → VaultLocked
    config: RwLock<AppConfig>, ui_prefs: RwLock<UiPreferences>,
    protocols: ConnectorRegistry, importers: ImportRegistry,
    sessions: SessionManager, watcher: Mutex<Option<DocumentWatcher>>,
    pending_deletes: Mutex<HashMap<UndoId, PendingDelete>>,
}
```
Un solo `Mutex` su documento + store. Nessun guard attraverso un `.await`. Un `Mutex` avvelenato si recupera come in
`LazyVault`. **Preferenze dell'interfaccia** (tema fisso, densità, sfondo animato, riduci movimento, ultimo documento,
documenti recenti, dimensioni della finestra) in un file separato `ui.toml` nella directory dei dati, non in
`config.toml` (DEC-011).

### 4.2 Errori IPC
`IpcError { code: ErrorCode, message: String, details: Option<IpcErrorDetails> }` serializzato, con `code` da
`OpsError::category()` e dagli errori dell'app: `NotFound`, `Ambiguous` (con candidati), `Invalid` (con campo),
`Referenced` (con riferimenti), `VaultLocked`, `Vault`, `ModifiedExternally`, `DocumentMissing`, `NoDocument`,
`Connect`, `Io`, `Internal`. `message` in italiano, mai segreti. Il frontend sceglie la presentazione dal `code`.

### 4.3 Comandi
Raggruppati per area; firme definitive scelte in UI-WP2 e generate in `bindings.ts`. Ogni comando che modifica
restituisce i dati aggiornati o un'invalidazione, e passa sempre da `portolano-ops`.
- **Documento**: `app_status` (documento aperto, percorso, vault, preferenze); `create_document` / `open_document`
  (percorso scelto con dialogo nativo da Rust: `choose_document`, `choose_new_document`); `recent_documents`;
  `reload_document`; `close_document`.
- **Albero e dettaglio**: `tree` (DTO con id, nome, tipo, protocollo, riepilogo, colore, icona, preferito, tag, stato
  dei segreti, figli); `node_details` (campi, credenziale effettiva con `resolve_trace`, stato dei segreti, utilizzi,
  sorgente d'import, `extra` in sola lettura per i non supportati); `search`.
- **Modifiche**: `add_node`, `edit_node` (con segreti in staging), `move_node`, `duplicate_node`,
  `preview_delete` → `delete_node` (con annullamento) → `undo_delete` / scadenza automatica.
- **Segreti**: `secret_status`, `set_secret` (password in ingresso), `clear_secret`.
- **Vault**: `vault_status`, `unlock_vault(master_password)`, `lock_vault`, `create_file_vault(master_password)`.
- **Validazione**: `validate` (composto, DEC-022.4).
- **Import**: `choose_import_file`; `import_preview(options)` → report + richieste di segreti;
  `import_apply(options, answers)` con le risposte nell'ordine delle richieste (verificate per percorso e campo);
  `save_import_report` (percorso da dialogo nativo).
- **Connessione**: `connect_preview(node)` (`render` del piano, avvisi); `login_request(node)`;
  `open_session(node, options, login?)` → `SessionId`; `session_input`, `session_resize`, `session_close`;
  `open_in_terminal(node)` (nuova finestra esterna).
- **Config e preferenze**: `config_get`, `config_set` (DEC-008/011), `ui_prefs_get`, `ui_prefs_set`.
- **Icone**: `choose_icon_file` → `add_icon`; `custom_icons`; `builtin_icons` (elenco dei nomi).

### 4.4 Eventi e canali
- **Eventi** (piccoli, rari): `document-changed` (dopo un salvataggio dell'app), `document-modified-externally`,
  `document-missing`, `vault-locked`, `session-ended`.
- **Canali** (`tauri::ipc::Channel`): output delle sessioni come **byte grezzi** (`InvokeResponseBody::Raw`, con un
  header versionato `{version, kind}` già pensato per i frame RDP futuri) e un canale JSON per lo stato della sessione
  (`connecting | connected | ended { code } | failed { code, reason }`).
- **Osservazione del file** (`notify` sulla directory padre, debounce 400 ms, poi `Store::changed_on_disk`): se il file
  è cambiato da fuori → evento; la UI propone "Ricarica" e, se l'editor è aperto, tiene le modifiche come patch e le
  riapplica dopo il ricaricamento (avvisa se un campo è stato cambiato da entrambe le parti).

### 4.5 Sessioni (modello indipendente dal protocollo)
`SessionManager` con id, tipo (`terminal | rdp-external | external`), stato e chiusura; un trait per i tipi di
sessione, così il pannello IronRDP futuro sarà un tipo in più (`rdp-embedded`) senza cambiare tab, canali e comandi.
- **SSH integrato (`terminal`)**: `plan` → `prepare` (DEC-016.2) → PTY con `portable-pty` (`TERM=xterm-256color`,
  `CHILD_ENV_REMOVE` tolte, env askpass del `PreparedLaunch`); la sessione **possiede** il `PreparedLaunch` finché
  `ssh` non esce (server askpass vivo, segreti azzerati alla fine). Thread di lettura (blocchi da 32 KiB sul canale
  binario) e di attesa (codice d'uscita; 255 = connessione non riuscita). Il fallback askpass su `/dev/tty` deve
  comparire nel terminale integrato (conferma della host key, prompt non riconosciuti): da verificare in UI-WP0.
  `Environment.self_exe` = percorso del sidecar.
- **RDP (`rdp-external`)**: FreeRDP in finestra propria. L'app lancia il processo **senza** staccarlo, in un thread che
  ne attende la fine, per mostrare lo stato nella tab (in corso, durata, chiusa, errore con "Mostra log").
  **DEC-025 dentro l'app**: quando il piano chiede il primo piano perché FreeRDP deve chiedere una password, FreeRDP
  si avvia **dentro una tab terminale** (PTY), così il prompt compare in xterm.js.
- **Windows App (`external`)**: `open -a`; la tab mostra "Aperta in Windows App" e si può chiudere, senza stato.
- **Nuova finestra**: `open_in_terminal` usa il piano `--new-window` con `self_exe` = sidecar.
- "Porta in primo piano" del mock **non** si implementa (richiederebbe i permessi di Accessibilità).

### 4.6 File e dialoghi
Tutti i percorsi scelti dall'utente passano da dialoghi nativi aperti da Rust (`tauri-plugin-dialog`, filtri per
estensione); il comando restituisce al frontend solo il nome da mostrare e conserva il percorso nello stato. Il
documento resta `*.json` (default `portolano.json`, ambiguità 16); l'associazione dei file è fuori scope.

### 4.7 Icone
- **Builtin**: set Lucide (licenza ISC, incluso nel bundle) con un sottoinsieme curato e nominato (server, desktop,
  database, terminale, cloud, rete, chiave, cartella, …); `Icon::Builtin { name }` usa questi nomi.
- **Royal** (`Icon::Royal`): tabella di corrispondenza dalle categorie più comuni della libreria Royal ai nomi
  builtin; senza corrispondenza si usa l'icona del tipo di nodo. Le immagini di Royal non si copiano (licenza).
- **Personalizzate**: schema `portolano-icon://localhost/<sha256>.<ext>` registrato in Rust; accetta solo nomi
  `^[0-9a-f]{64}\.(png|jpg|gif|tiff|svg|pdf)$`, legge solo dalla directory icone del documento aperto, imposta
  `Content-Type`, `X-Content-Type-Options: nosniff` e per gli SVG una CSP `default-src 'none'`. TIFF e PDF: verificare
  in UI-WP0 che WKWebView li mostri in `<img>`, altrimenti segnaposto.

### 4.8 Sicurezza Tauri
CSP: `default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'` (da provare a togliere
`'unsafe-inline'` in UI-WP0); `img-src 'self' data: blob: portolano-icon:`; `font-src 'self'`;
`connect-src ipc: http://ipc.localhost`; `object-src 'none'; base-uri 'none'; form-action 'none'; frame-src 'none'`.
`freezePrototype: true`, `assetProtocol` disattivato, `withGlobalTauri: false`. Capability `main` con i soli permessi
necessari (`core:event`, `core:window` di base e `allow-start-dragging`, dialog, `opener:allow-open-url` limitato a
`http(s)`, e un `allow-<comando>` per ogni comando generato da `AppManifest`). Terminale: titolo OSC solo come testo,
link solo `http(s)` aperti con `opener`, clipboard OSC 52 disattivato. Isolation pattern: rimandato (annotato).

### 4.9 Finestra e menu
Finestra unica `main`, 1280×800, minima 900×560, `titleBarStyle: Overlay`, titolo nascosto, semafori nativi
(posizione `x:16, y:18`), barra disegnata dalla UI con `data-tauri-drag-region`. Finestra **opaca**: lo sfondo è la
costellazione del mock su canvas (niente vibrancy nativa). Dimensione e posizione ricordate. **Menu nativo** (in
italiano): Portolano (Informazioni, Preferenze… ⌘,, Esci), File (Nuovo documento, Apri… ⌘O, Documenti recenti,
Importa… ⌘I, Chiudi tab ⌘W), Modifica (standard), Nodo (Nuova connessione ⌘N, Nuova cartella ⇧⌘N, Nuova credenziale,
Connetti ⌘↩, Modifica ⌘E, Duplica ⌘D, Elimina ⌘⌫), Vista (Cerca ⌘F, Valida ⇧⌘V, Densità), Finestra (standard).

### 4.10 Sidecar e build
- In sviluppo: `beforeDevCommand` compila anche `portolano-cli`; a runtime il sidecar è
  `current_exe().parent().join("portolano")` (in `target/debug` accanto all'app); se manca → errore chiaro.
- Nel bundle: `externalBin` solo in `tauri.bundle.json` usato da `npm run bundle` (script che compila la CLI in
  release e la copia in `binaries/portolano-<triple>`), così `cargo build/clippy/test --workspace` non ne dipendono.
  Il binario dell'app si chiama `portolano-app` (mai `Portolano`: collisione su APFS).
- Firma ad hoc (`signingIdentity: "-"`), `hardenedRuntime`; Developer ID e notarizzazione fuori scope.

---

## 5. Frontend

### 5.1 Struttura e stato
`app/` (provider, routing interno, scorciatoie, ascolto degli eventi), `ipc/` (wrapper tipizzati su `bindings.ts`,
mappa degli errori, **trasporto delle sessioni dietro un'interfaccia** per poterlo simulare nei test), `state/`
(Zustand: selezione, tab e sessioni aperte, pannelli, vault, toast, preferenze UI), `design/` (token, font, sfondo,
motion), `components/` (catalogo §5.3), `features/` (tree, details, editor, dialogs, sessions, import, validation,
preferences, first-run), `i18n/` (testi italiani centralizzati, singolare/plurale), `test/` (utilità, finto backend),
`build/` (plugin del solo build di Vite, fuori dal bundle; DEC-034).
**TanStack Query** per i dati del backend (chiavi per albero, dettaglio, validazione, config; invalidazione dopo le
mutazioni e sugli eventi); **Zustand** per lo stato dell'interfaccia. Sessioni:
`Session { id, nodeId, kind: 'terminal' | 'rdp-external' | 'external' | 'rdp-embedded', state, startedAt, exitCode? }`
e un registro `sessionViews[kind] → componente` (la tab non conosce xterm né canvas).

### 5.2 Design system
- **Token** come CSS custom properties generate da un unico modulo TypeScript (`design/tokens.ts`), con i valori
  esatti di `inventario.md` §2 e nomi semantici (ambiguità 1): `--color-text-primary`, `--color-text-secondary`
  (`#CFC9BD`), `--color-text-muted` (`#A8A297`), superfici, vetro, accento, semantici, tipi di nodo; scala di raggi con
  il 14 px (`--radius-panel`), spaziature, ombre, blur, durate e curve; z-index (proposta dell'inventario).
- **Contrasto** (inventario §7): i valori che non raggiungono AA vengono corretti mantenendo la tinta: testo su accento
  (riempimento più scuro, hover più scuro e non più chiaro), `--err` per il testo (verso `#EE8279`), testo "non
  supportato", bordi dei campi e binario dei toggle spenti (≥ 3:1). Un **test automatico** calcola i rapporti WCAG
  delle coppie usate e fallisce sotto soglia.
- **Font**: Hanken Grotesk (400/500/600) e IBM Plex Mono da `@fontsource`, inclusi; verificare `tabular-nums` (se
  manca, colonne numeriche in Plex Mono).
- **Sfondo**: costellazione su canvas come da inventario §3.1, ~30 fps, solo `transform`/`opacity` nei layer CSS,
  sospesa con finestra non attiva o nascosta, più calma con un dialogo aperto (velo al 45%, ambiguità 4) e mai sotto il
  terminale. Preferenza "Sfondo animato: acceso / ridotto / spento"; intensità predefinita 8 (ambiguità 19).
- **Riduci movimento** (preferenza + `prefers-reduced-motion`): sfondo statico, spostamenti sostituiti da dissolvenze
  ≤ 120 ms (ambiguità 5).
- **Densità**: normale e compatta (righe 26 px e testo 12.5 px, ambiguità 8).
- **Galleria** (`?gallery`, solo in sviluppo e nei test): tutti i componenti in tutti gli stati, per il checkpoint
  visivo e gli screenshot.

### 5.3 Componenti
Il catalogo di `inventario.md` §4 (~50 componenti, con varianti, stati e misure), costruiti su elementi semantici
(`button`, `input`, `label`) o con ruoli ARIA completi: `Button` (disattivato come nel DS, ambiguità 7), `TextField`,
`TextArea`, `Select`, `Toggle` (`role=switch`), `SegmentedControl` (`radiogroup`), `FilterChip`, `Tabs` (`tablist`),
`SecretBadge` (presente / assente / da salvare / vault bloccato, ambiguità 14), `StatusBadge`, `SeverityMarker`,
`Tag`, `StatusDot` (con testo alternativo), `Tooltip`, `ContextMenu`, `Dialog` (`role=dialog`, focus trap, Esc),
`SidePanel`, `Toast` (`aria-live`), `TreeRow` (Headless Tree, `role=tree`), `SortableList` (jump host, riordinabile
anche da tastiera), `RouteChain`, `SessionRoute`, `CredentialField`, `CredentialSelector` (**dialogo modale**,
ambiguità 6) con `CredentialPreview`, `Stepper`, `IconPicker`, `ColorPicker` (6 campioni + "Nessuno" +
"Personalizzato…" esadecimale, ambiguità 15), `NodePicker`, `Terminal` (xterm.js, ambiguità 24).

### 5.4 Schermate e flussi
Come `inventario.md` §5 (testi esatti, layout, azioni), con le correzioni marcate **[CORREZIONE]** e le decisioni
di §6. In sintesi:
- **Finestra principale**: barra con "+ Connessione" (menu SSH/RDP), "+ Cartella", "+ Credenziale", Importa, Valida,
  Preferenze; albero con ricerca (nome, host, tag) e filtri (SSH, RDP, Credenziali, ★), drag & drop per spostare e
  riordinare (`move_node`), menu contestuale, conteggio dei discendenti sulle cartelle chiuse (ambiguità 21), colore
  del nodo come pallino prima del nome; piè del vault (§6, ambiguità 20); area a tab con dettaglio e sessioni.
- **Dettaglio**: riepilogo, credenziale effettiva con la catena (`resolve_trace`), segreti (stato), utilizzi,
  "Mostra comando" (`connect_preview`: il piano reale, con `<vault:…>`), note, proprietà, sorgente d'import; nodo non
  supportato in sola lettura (niente "Modifica").
- **Editor** (pannello laterale in due passi): Generale (nome, posizione con "Cambia…", descrizione, note, icona,
  colore, preferito, tag, proprietà) e Connessione/Accesso/Credenziale predefinita per tipo; **Salva disponibile in
  entrambi i passi**; segreti in staging fino al Salva; porta lasciata al predefinito = nessuna porta; opzioni ssh solo
  quelle ammesse (DEC-026.1) con il messaggio del backend sul campo; nessun valore d'esempio precompilato.
- **"+ Connessione"** (ambiguità 10): sceglie il protocollo dal menu; la destinazione è la cartella selezionata (o la
  cartella del nodo selezionato, o la radice) ed è modificabile al passo 1. Lo stesso per cartelle e credenziali.
- **Dialoghi**: selettore di credenziale (6 modalità; "Per nome" cerca in tutto il documento; stati "nessun
  risultato" e "ambiguo" disegnati, ambiguità 23), icona, picker di nodi, password (titolo con campo e nodo; bottone
  "Imposta"), sblocco e creazione del vault su file (minimo 12 caratteri, master password errata), credenziali
  all'avvio per "Chiedi all'avvio" (DEC-016.3), conferma eliminazione (riferimenti diretti + "la ricevono per
  ereditarietà N nodi", contenuto e segreti di una cartella, "Elimina comunque"), primo avvio (nuovo documento, apri,
  recenti; niente "cartella condivisa").
- **Eliminazione annullabile**: toast "Eliminato · Annulla" per 10 s; i segreti si cancellano alla scadenza.
- **Sessioni**: tab SSH (terminale, stato unico: in corso / connesso al primo output / terminata / non riuscita con
  codice, "Riprova"; ambiguità 11); tab RDP (stato del processo FreeRDP, durata che scorre, "Mostra log" in caso
  d'errore, "Disconnetti"); tab Windows App; "Apri in terminale esterno" dal menu.
- **Wizard d'import**: file (dialogo nativo; formati `.rtsz`, `.rtsx`, XML anche compressi), opzioni (destinazione con
  picker, conflitti, cestino, chiedi le password), password una alla volta con "N / M", "Salta", "Salta tutte le
  restanti" (dalle richieste di `import_preview`), report (conteggi, elementi con motivo, clic → nodo tramite
  `source_id`, "Esporta JSON"). Una password saltata è "Parziale"; "Saltati" solo per il cestino.
- **Validazione**: tab con righe su due righe (ambiguità 9), gravità Errore/Avviso/Nota, "Vai al nodo" su ogni riga,
  conteggio reale, "Ricontrolla"; con vault bloccato una sola nota invece di un `missing_secret` per password.
- **Preferenze**: documento predefinito, vault (portachiavi / file cifrato con percorso), terminale esterno
  (Terminal, iTerm, Ghostty, comando con `{cmd}`), programma SSH, RDP (client, percorso FreeRDP, certificati), aspetto
  (densità, sfondo animato, riduci movimento). Le voci di config passano da `config_set` (validate dal backend).
- **Documento modificato all'esterno / mancante**: banner con "Ricarica" o "Ricrea / Apri altro".
- **Caricamenti ed errori**: skeleton per l'albero e il dettaglio, spinner sui bottoni durante i comandi, toast
  d'errore persistenti con chiusura (in basso a destra, 360 px, al massimo 3; ambiguità 17), errori dei campi sotto i
  campi.

### 5.5 Tastiera
Navigazione completa da tastiera: albero (frecce, Home/End, Invio = apri dettaglio, ⌘↩ = connetti, typeahead), tab
(⌘1…⌘9, ⌘W), dialoghi (Esc, Invio sull'azione predefinita, ma Invio = Annulla nella conferma d'eliminazione),
scorciatoie del menu (§4.9). Focus visibile su tutti gli elementi interattivi.

---

## 6. Decisioni sulle ambiguità del mock (inventario §9)

| # | Decisione |
|---|---|
| 1 | Nomi semantici dei token (§5.2), indipendenti dai nomi del mock. |
| 2 | Il raggio 14 px entra nella scala come `--radius-panel`. |
| 3 | Logo: variante della tavola App; l'icona dell'app si ricava da quel logo (SVG → `tauri icon`) in UI-WP1. |
| 4 | Velo dei dialoghi 45%; con un dialogo aperto lo sfondo rallenta al 25%. |
| 5 | Riduci movimento: dissolvenze ≤ 120 ms al posto degli spostamenti, sfondo statico. |
| 6 | Selettore di credenziale come dialogo modale. |
| 7 | Bottone disattivato come nel DS (`--field` + testo muted), con `aria-disabled`. |
| 8 | Densità compatta: righe 26 px e testo 12.5 px. |
| 9 | Righe di validazione su due righe (tavola App). |
| 10 | "+ Connessione": menu SSH/RDP; destinazione dalla selezione, modificabile nell'editor. |
| 11 | Stato unico di connessione: in corso → connesso (primo output) → terminata / non riuscita (codice). |
| 12 | Tab RDP: stato del processo, durata, log; FreeRDP che deve chiedere → tab terminale; niente "Porta in primo piano". |
| 13 | Eliminazione: riferimenti diretti (DEC-018.1) più il conteggio di chi la riceve per ereditarietà. |
| 14 | Segreto in staging: badge "Da salvare"; bottone del dialogo password "Imposta". |
| 15 | Colore: pallino prima del nome nell'albero e tessera dell'icona nel dettaglio; 6 campioni + Nessuno + Personalizzato. |
| 16 | Documento `*.json` (default `portolano.json`); niente estensione `.portolano`. |
| 17 | Toast d'errore persistenti con "×", in basso a destra, 360 px, massimo 3; info 5 s; annullamento 10 s. |
| 18 | Finestra 1280×800, minima 900×560, barra `Overlay` con semafori nativi. |
| 19 | Solo palette 2a; sfondo con intensità 8 e preferenza acceso/ridotto/spento. |
| 20 | Piè del vault: "Portachiavi di sistema" senza stato; per il file "Vault su file · bloccato/sbloccato" cliccabile. |
| 21 | Cartella chiusa: conteggio dei discendenti nella meta. |
| 22 | `tabular-nums` da verificare sul file di Hanken Grotesk; altrimenti Plex Mono per i numeri. |
| 23 | Stati "nessuna credenziale ereditata", "nome non trovato", "nome ambiguo" disegnati nel `CredentialPreview`. |
| 24 | Terminale con xterm.js 6 (renderer DOM, WebGL facoltativo), font IBM Plex Mono, colori dai token. |

Gli elementi del mock da **non** implementare sono in `inventario.md` §8.

---

## 7. Test e quality gate

**Rust** (invariati da SPEC §11.2, estesi al nuovo crate): `cargo fmt --all --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`. La logica dei comandi si testa
fuori da Tauri (funzioni che ricevono lo stato); la serializzazione di `IpcError` e i permessi con `tauri::test`.
Sessioni PTY testate con programmi sonda innocui al posto di `ssh` (come nei connettori). `scripts/smoke.sh` resta verde.

**Frontend** (in `frontend/`, Node 22.23.2): `npm run typecheck` (`tsc --noEmit`, `strict`), `npm run lint`
(ESLint con `recommendedTypeChecked`, `--max-warnings 0`), `npm run format:check` (Prettier), `npm test` (Vitest +
Testing Library con IPC simulato: `mockIPC` e il trasporto delle sessioni finto), `npm run e2e` (Playwright **WebKit**
sul frontend con un finto backend in memoria: flussi principali e screenshot della galleria e delle schermate),
`npm run build`. `bindings.ts` rigenerato e senza differenze (`npm run bindings:check`).

**Standard del TypeScript**: componenti funzionali piccoli (un file per componente, < ~250 righe), hook per la logica,
nessuna logica di dominio (sta in Rust), nessun `any`, `as` solo motivato, nessun `eslint-disable` senza commento,
niente segreti in store o log, testi solo da `i18n/`, CSS con i token (niente colori o misure letterali fuori da
`design/`). Test: un comportamento per test, query per ruolo e testo accessibile.

**Segreti**: un test e2e e un test Rust verificano con un segreto sentinella che nessuna risposta IPC, evento, canale,
log o file dell'app lo contenga.

---

## 8. Pacchetti di lavoro

| WP | Contenuto | Proprietà dei file | Dipende da | Ondata |
|---|---|---|---|---|
| UI-WP0 | **Prototipo e scheletro**: modifiche ai crate §3.1–3.5; crate `portolano-app` e `frontend/` con versioni, lint, test, CSP, capability, AppManifest, bindings, schema icone, sidecar in dev, finestra; un comando e una schermata di prova; chiusura dei punti incerti della ricognizione (sidecar e `--config`, askpass su `/dev/tty` nella PTY, `unsafe-inline`, TIFF/PDF in `<img>`, permessi dei plugin, prestazioni del vetro, dimensione e Keychain del bundle) con esito in `DECISIONS.md` tramite il PM | crate citati (solo le parti §3.1–3.5), `crates/portolano-app/**`, `frontend/**`, `Cargo.toml` radice | – | A |
| UI-WP1 | **Design system**: token, font, contrasto (con test), sfondo, motion, tutti i componenti di §5.3 con test, galleria e screenshot Playwright | `frontend/src/design/**`, `frontend/src/components/**`, `frontend/src/i18n/**` (base), test relativi | WP0 | B |
| UI-WP2 | **Backend**: §3.6 in ops (con test), stato, tutti i comandi di §4.3 tranne le sessioni, eventi, watcher, icone, preferenze UI, menu; bindings | `crates/portolano-ops/**` (aggiunte §3.6), `crates/portolano-app/src/**` esclusa `sessions/` | WP0 | B |
| UI-WP3 | **Sessioni**: `SessionManager`, PTY, FreeRDP, Windows App, nuova finestra (backend) + tab, terminale, stati (frontend) | `crates/portolano-app/src/sessions/**`, `frontend/src/features/sessions/**` | WP0 (frontend: WP1) | B |
| UI-WP4 | **Finestra principale**: albero, ricerca e filtri, drag & drop, menu contestuale, dettaglio, eliminazione con annullamento, duplicazione, piè del vault, banner, primo avvio | `frontend/src/features/{tree,details,first-run}/**`, `frontend/src/app/**` | WP1, WP2 | C |
| UI-WP5 | **Editor e dialoghi**: editor in due passi, segreti in staging, selettori, password, vault, credenziali all'avvio, preferenze | `frontend/src/features/{editor,dialogs,preferences}/**` | WP1, WP2 | C |
| UI-WP6 | **Import e validazione** | `frontend/src/features/{import,validation}/**` | WP1, WP2 | C |
| UI-WP7 | **Bundle e chiusura**: `npm run bundle` (sidecar in release, firma ad hoc), README (sezione app), `progress.md`, e2e completi; in parallelo **revisione di sicurezza** dell'app | file indicati | tutti | D |

Ondata B: WP1, WP2 e WP3 in parallelo; ondata C: WP4, WP5, WP6 in parallelo. Merge nell'ordine della tabella, con i
quality gate (Rust e frontend) dopo ogni merge. **Checkpoint con l'utente**: dopo UI-WP1 (screenshot della galleria
dei componenti) e dopo l'ondata C (screenshot di tutte le schermate). Prima di ogni checkpoint l'app deve girare con
`npm run tauri dev`.

---

## 9. Definition of Done della FASE 2

- [ ] Quality gate Rust e frontend verdi su un checkout pulito; `scripts/smoke.sh` → `SMOKE OK`.
- [ ] `npm run bundle` produce `Portolano.app` (firma ad hoc) con il sidecar `portolano`; l'app si apre dal Finder.
- [ ] Tutte le schermate di §5.4 implementate con l'aspetto del mock (approvate dall'utente ai checkpoint) e le
      correzioni di `NOTE.md`; contrasto AA verificato dal test; navigazione da tastiera completa.
- [ ] Flussi verificati da e2e (finto backend): crea/apri documento, albero e drag & drop, editor con segreti in
      staging, duplicazione, eliminazione con annullamento, import con password, validazione, preferenze.
- [ ] Sessione SSH nel terminale integrato verificata con un programma sonda (test) e **dall'utente su un host reale**
      (password salvata, conferma della host key nel terminale); RDP con FreeRDP: stato nella tab e autologon
      **verificati dall'utente**.
- [ ] Nessun segreto verso il frontend (test con sentinella); revisione di sicurezza dell'app senza finding aperti.
- [ ] README con la sezione dell'app (installazione, uso, sicurezza, limiti); `progress.md` aggiornato.

## 10. Fuori scope

RDP integrato (IronRDP: dopo, nello spazio predisposto), più documenti aperti insieme, associazione dei file, firma
Developer ID e notarizzazione, aggiornamenti automatici, Windows e Linux (devono solo compilare, se possibile),
mobile, isolation pattern di Tauri (annotato come miglioramento), "utente proposto" per "Chiedi all'avvio", "Porta
in primo piano" della tab RDP.
