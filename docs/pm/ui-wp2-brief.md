# Brief UI-WP2 – Comandi del backend (ondata B)

Sei lo sviluppatore di **UI-WP2** di Portolano (app desktop Tauri v2 + React + TypeScript). Lavori nel tuo worktree
(directory corrente). Le regole comuni sono in `docs/pm/brief-common-ui.md`: leggile per prime e rispettale tutte.

## Commit di partenza
`3c710d1` su `main` (merge di UI-WP0). Primo passo: `git log --oneline -3`; se il commit manca,
`git merge --ff-only main`.

## Obiettivo
Il backend dell'app tranne le sessioni: le aggiunte a `portolano-ops` di SPEC-UI §3.6, lo stato completo (§4.1),
gli errori IPC (§4.2), **tutti i comandi di §4.3 tranne quelli delle sessioni**, gli eventi (§4.4), l'osservazione
del file, le icone (§4.7), le preferenze dell'interfaccia, il menu nativo (§4.9), i bindings TypeScript e il **finto
backend** in memoria che i WP di frontend useranno nei test e negli e2e.

## Fonti (oltre a quelle del brief comune)
- `docs/SPEC-UI.md` per intero, in particolare §1, §2, §3.6, §4 (tranne §4.5, salvo per capire i confini), §5.4 (per
  sapere che cosa servirà alle schermate), §7.
- `docs/SPEC.md` §1, §3–§6, §9, §10 (comportamenti di `portolano-ops` che i comandi espongono), §11.3.
- `docs/DECISIONS.md`: tutte; in particolare DEC-011, DEC-014, DEC-016, DEC-018, DEC-022, DEC-024, DEC-026, DEC-028,
  DEC-029 (proprietà dei file dell'ondata B).
- `docs/progress.md`, sezione "Esigenze della UI (FASE 2) aggiungibili senza rompere il core".
- Il codice: `crates/portolano-ops` (API esistente: è il contratto, la estendi senza romperla), `crates/portolano-app`
  di UI-WP0 (`commands/`, `error.rs`, `state.rs`, `icons.rs`, `bindings.rs`, `build.rs`, `capabilities/main.json`),
  `frontend/src/ipc/**`, `frontend/src/test/fakeBackend.ts`.

## File di tua proprietà
- `crates/portolano-ops/**` per le aggiunte di §3.6 (con test), e in `crates/portolano-core/src/icons*` solo
  `IconStore::list()` e il limite di dimensione (compito 6);
- `crates/portolano-app/**` **esclusa** `crates/portolano-app/src/sessions/**` e `crates/portolano-app/icons/**`
  (quest'ultima è di UI-WP1): `src/` (stato, errori, DTO, comandi, eventi, watcher, preferenze, menu), `build.rs`,
  `capabilities/`, `permissions/`, `tauri.conf.json`, `Cargo.toml` dell'app;
- `frontend/src/bindings.ts` (generato), `frontend/src/ipc/**` (wrapper tipizzati, query/mutazioni di TanStack Query,
  mappa degli errori) **escluso** il trasporto delle sessioni (`frontend/src/ipc/sessions*`, di UI-WP3);
- `frontend/src/test/fakeBackend*.ts` (il finto backend; le sessioni finte sono di UI-WP3);
- `frontend/src/i18n/ipc.ts` (testi dell'area IPC, se servono) e una riga in `frontend/src/i18n/it.ts` (DEC-029).

Punti d'integrazione condivisi con UI-WP3 (DEC-029): `commands/mod.rs` (`collect_commands!`), `build.rs`
(`COMMANDS`), `capabilities/main.json`, `state.rs`, `lib.rs`. Sono tuoi; UI-WP3 vi aggiungerà le proprie righe dopo
il tuo merge. Lascia in `AppState` e in `lib.rs` un punto naturale per le sessioni (es. il campo
`sessions: SessionManager` lo aggiungerà UI-WP3: non crearne uno finto).

## Compiti
1. **Aggiunte a `portolano-ops`** (§3.6), ciascuna con test e doc comment, senza stampe né TTY (DEC-018.2, DEC-022):
   `resolve_trace`, `credential_usage` (diretti DEC-018.1 + ereditati), `duplicate_node` (id e `SecretRef` nuovi,
   valori copiati nel vault, DEC-016.4), eliminazione annullabile (`delete` con cancellazione dei segreti rimandata
   che restituisce sottoalbero e `SecretRef`; `restore_deleted`; `purge_secrets`), `import_requests` (prima fase in
   prova, senza scrivere), `IconStore::list()`, **segreti in staging** in `add_node`/`edit_node` (nel percorso di
   salvataggio unico segreti → icone → documento con rollback; sostituzione con `SecretRef` nuovo e vecchio cancellato
   dopo il salvataggio; "Cancella" come modifica applicata al salvataggio). Il comportamento della CLI non cambia.
2. **Stato** (§4.1): `AppState` con documento aperto (un solo `Mutex` su store + documento, nessun guard attraverso un
   `.await`, mutex avvelenato recuperato come in `LazyVault`), vault (`LazyVault` + `UnlockSlot`), config, preferenze
   UI in `ui.toml` nella directory dei dati (non in `config.toml`, DEC-011), registry, watcher, eliminazioni in
   sospeso con scadenza (10 s, poi `purge_secrets`).
3. **Errori IPC** (§4.2): completa `IpcError` e i `details` (candidati, campo, riferimenti), messaggi italiani senza
   segreti; usa `NoDocument` (togli l'`expect(dead_code)` di UI-WP0).
4. **Comandi** (§4.3), adattatori sottili sopra `portolano-ops`, ciascuno con test della logica fuori da Tauri
   (funzioni che ricevono lo stato) e un permesso `allow-<comando>` (`build.rs` + capability): documento, albero e
   dettaglio (DTO dell'albero con id, nome, tipo, protocollo, riepilogo, colore, icona, preferito, tag, stato dei
   segreti, figli; `node_details` con `resolve_trace`, utilizzi, sorgente d'import, `extra` in sola lettura),
   `search`, modifiche (`add_node`, `edit_node` con staging, `move_node`, `duplicate_node`, `preview_delete` →
   `delete_node` → `undo_delete`), segreti, vault, `validate` (con la sola nota a vault bloccato, §5.4), import
   (`choose_import_file`, `import_preview` con richieste, `import_apply` con risposte verificate per percorso e campo,
   `save_import_report`), **connessione solo** `connect_preview` e `login_request` (le sessioni, `open_session`,
   `session_*` e `open_in_terminal` sono di UI-WP3), config e preferenze, icone (`choose_icon_file` → `add_icon`,
   `custom_icons`, `builtin_icons`). Percorsi solo da dialoghi nativi aperti da Rust (§4.6): al frontend il nome da
   mostrare, il percorso resta nello stato. Ogni comando che modifica restituisce i dati aggiornati o
   un'invalidazione. Le password in ingresso arrivano in **un solo** comando e diventano subito `Secret`
   (SPEC-UI §1.2); nessun comando restituisce segreti (§1.1).
5. **Eventi e watcher** (§4.4): `document-changed`, `document-modified-externally`, `document-missing`,
   `vault-locked` (`session-ended` è di UI-WP3); `notify` sulla directory padre con debounce 400 ms, poi
   `Store::changed_on_disk`. Eventi tipizzati con tauri-specta.
6. **Icone** (§4.7): i nomi dei builtin sono **esattamente** quelli di DEC-029.4; nel backend sono la fonte per
   `builtin_icons` e per la tabella di corrispondenza delle icone Royal (categorie più comuni → nome builtin; senza
   corrispondenza l'icona del tipo di nodo), che il DTO dell'albero e del dettaglio applica già. Esportali anche come
   costante nei bindings (tauri-specta), così il frontend li confronta con la mappa di UI-WP1: prima della tua
   revisione finale il PM ti chiederà di fare `git merge main` (con UI-WP1 già integrato) e di aggiungere in
   `frontend/src/ipc/` un test che verifica che la mappa di UI-WP1 copra tutti e soli i nomi della costante.
   Icone personalizzate: lo schema è già fatto in UI-WP0, ma il suo limite `MAX_ICON_BYTES` (4 MiB) esiste solo
   nell'app, mentre `IconStore::add` accetta qualsiasi dimensione: un'icona più grande verrebbe salvata e poi mai
   mostrata. Sposta il limite in `portolano-core::icons` come costante pubblica, fallo rifiutare da `IconStore::add`
   con un errore chiaro (test) e riusalo nello schema dell'app (finding della revisione di UI-WP0).
7. **Menu nativo** (§4.9) in italiano, con le scorciatoie; le voci emettono un evento tipizzato verso il frontend
   (l'azione la esegue l'interfaccia dell'ondata C).
8. **Bindings**: `npm run bindings` e `npm run bindings:check` verdi; il test che esporta tutti i tipi di §3.5
   (`bindings.rs`, elenco scritto a mano) resta, e ogni tipo a cui aggiungi il derive `specta` (es. `EditOutcome`,
   `DeleteOutcome`, con i contatori annotati `specta(type = u32)`) va aggiunto a quell'elenco.
9. **Frontend IPC**: in `frontend/src/ipc/` wrapper tipizzati, chiavi e opzioni di TanStack Query (albero,
   dettaglio, validazione, config, preferenze) e invalidazione dopo le mutazioni e sugli eventi, mappa
   `IpcError.code` → presentazione (testi in `i18n/ipc.ts`); test Vitest con `mockIPC`. Nessun segreto in store,
   cache di Query o log: le mutazioni con password non tengono la password nelle variabili di Query
   (documentalo nel codice e con un test).
10. **Finto backend** (`frontend/src/test/fakeBackend*.ts`): un documento in memoria con dati finti (RFC 5737,
    `*.example.com`) che risponde a **tutti** i tuoi comandi con la stessa semantica del Rust (errori compresi),
    usato da Vitest e dagli e2e (`main.tsx` in modalità `e2e` lo installa già). Diviso in moduli piccoli. È ciò su
    cui l'ondata C costruirà i propri test.
11. **Segreto sentinella** (§7): un test Rust che esegue i comandi con un segreto sentinella e verifica che nessuna
    risposta, evento, log o file dell'app lo contenga.

## Gate prima di consegnare (output reale nel report)
Rust: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
In `frontend/`: `npm run bindings:check`, `npm run typecheck`, `npm run lint`, `npm run format:check`, `npm test`,
`npm run build`, `npm run e2e`.
