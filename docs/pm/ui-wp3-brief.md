# Brief UI-WP3 – Sessioni (ondata B)

Sei lo sviluppatore di **UI-WP3** di Portolano (app desktop Tauri v2 + React + TypeScript). Lavori nel tuo worktree
(directory corrente). Le regole comuni sono in `docs/pm/brief-common-ui.md`: leggile per prime e rispettale tutte.

## Commit di partenza
`3c710d1` su `main` (merge di UI-WP0). Primo passo: `git log --oneline -3`; se il commit manca,
`git merge --ff-only main`.

## Obiettivo
Le sessioni dell'app (SPEC-UI §4.5, §5.1, §5.4 "Sessioni"): il `SessionManager` indipendente dal protocollo, il
terminale SSH integrato su PTY, FreeRDP in finestra propria con lo stato nella tab, Windows App, la nuova finestra
esterna; nel frontend le tab di sessione, il terminale e gli stati. Più la correzione **DEC-028.2** del client askpass.

## Due fasi (DEC-029)
UI-WP3 lavora in parallelo con UI-WP1 (componenti, fra cui `Terminal` e `Tabs`) e UI-WP2 (`AppState`, registrazione
dei comandi). Perciò:
- **Fase 1 (ora)**: tutto ciò che non dipende da loro (sotto, punti 1–5). Consegna con report di fase 1 e fermati.
- **Fase 2 (dopo il merge di UI-WP1 e UI-WP2 su `main`; il PM ti riprende con SendMessage)**: `git merge main` nel
  tuo branch, poi i comandi Tauri delle sessioni sopra l'`AppState` di UI-WP2 e le viste del frontend con i
  componenti di UI-WP1 (punti 6–8).

## Fonti (oltre a quelle del brief comune)
- `docs/SPEC-UI.md` per intero, in particolare §1, §4.3 ("Connessione"), §4.4 (canali), §4.5, §4.8 (terminale),
  §5.1, §5.4 ("Sessioni"), §6 (ambiguità 11, 12, 24), §7.
- `docs/SPEC.md` §1, §7 (connettori, askpass §7.2, piani §7.1–7.3), §11.3.
- `docs/DECISIONS.md`: tutte; in particolare DEC-009, DEC-013, DEC-016.2–3, DEC-020, DEC-023, DEC-025, DEC-027,
  **DEC-028.2**, DEC-029.
- `docs/ui/inventario.md` §3.3, §4.8, §4.22, §5.4 (tab SSH e RDP), §6; `docs/ui/ricerca-tauri.md` (PTY, canali
  binari, xterm.js).
- Il codice: `portolano-connectors` (`prepare`, `PreparedLaunch`, `CHILD_ENV_REMOVE`, `ssh/askpass/client.rs`),
  `portolano-ops::connect`, il test `crates/portolano-app/tests/askpass_tty.rs` di UI-WP0 (mostra già PTY + askpass).

## File di tua proprietà
- `crates/portolano-app/src/sessions/**` (e i suoi test; test d'integrazione in
  `crates/portolano-app/tests/sessions*.rs`);
- `crates/portolano-connectors/src/ssh/askpass/client.rs` (e i test dell'askpass) per DEC-028.2;
- `frontend/src/features/sessions/**`, `frontend/src/ipc/sessions*` (trasporto delle sessioni dietro
  un'interfaccia), `frontend/src/test/fakeSessions*.ts` (trasporto finto), `frontend/src/i18n/sessions.ts` e una
  riga in `frontend/src/i18n/it.ts`, `frontend/e2e/sessions*.spec.ts`;
- **solo in fase 2**, le righe d'integrazione delle sessioni in `crates/portolano-app/src/{lib.rs,state.rs,
  commands/mod.rs}`, `build.rs`, `capabilities/main.json` e la rigenerazione di `frontend/src/bindings.ts`.

## Compiti – fase 1
1. **DEC-028.2**: il client askpass, nel fallback su `/dev/tty` per i prompt senza eco, **spegne l'eco prima di
   scrivere il prompt** e la riaccende sempre (anche in errore o con un segnale gestibile: guard RAII). Il
   comportamento per la CLI resta identico. Test: il prompt compare solo quando l'eco è già spento (su una PTY,
   leggendo i termios dal master quando il prompt è visibile); aggiorna `crates/portolano-app/tests/askpass_tty.rs`
   solo se cambia ciò che asserisce (te ne è concessa la modifica per questo).
2. **`SessionManager`** (§4.5): id, tipo (`terminal | rdp-external | external`, con il posto per `rdp-embedded`),
   stato (`connecting | connected | ended { code } | failed { code, reason }`, ambiguità 11: connesso al primo
   output), chiusura; un trait per i tipi di sessione. Indipendente da Tauri dove possibile (canali astratti), così
   si testa senza finestra.
3. **Tipi di sessione**:
   - `terminal`: dal `PreparedLaunch` (DEC-016.2) a una PTY `portable-pty` con `TERM=xterm-256color`,
     `CHILD_ENV_REMOVE` tolte e l'env askpass del lancio; la sessione **possiede** il `PreparedLaunch` finché il
     processo non esce; thread di lettura (blocchi da 32 KiB) e di attesa (codice d'uscita; 255 = connessione non
     riuscita); input e ridimensionamento; `Environment.self_exe` = sidecar (`sidecar.rs` di UI-WP0).
   - `rdp-external`: FreeRDP **non** staccato, in un thread che ne attende la fine (in corso, durata, chiusa, errore
     con il log); quando il piano chiede il primo piano (DEC-025) FreeRDP gira **in una sessione `terminal`**.
   - `external` (Windows App): lancio e stato "aperta", senza processo da seguire.
   - nuova finestra esterna: il piano `--new-window` con `self_exe` = sidecar.
   - "Porta in primo piano" non si implementa (§4.5).
4. **Canali** (§4.4): output come **byte grezzi** con header versionato `{version, kind}` (pensato per i frame RDP
   futuri) e stato in JSON; evento `session-ended`. Nessun segreto nei canali (test con segreto sentinella).
5. **Frontend, parte senza componenti**: in `frontend/src/ipc/sessions*` l'interfaccia del trasporto (apri, input,
   resize, chiudi, flusso di byte e di stato) con l'implementazione sopra `Channel` di Tauri e una finta
   (`frontend/src/test/fakeSessions*.ts`) per Vitest ed e2e; in `features/sessions/` lo store Zustand delle sessioni
   (`Session { id, nodeId, kind, state, startedAt, exitCode? }`), il registro `sessionViews[kind] → componente`
   (la tab non conosce xterm né canvas) e la logica degli stati, con test. Testi in `i18n/sessions.ts`.

Test Rust con programmi sonda innocui al posto di `ssh` e FreeRDP (mai `ssh` reale, mai FreeRDP con argomenti di
connessione, mai `open` reale: per Windows App e la nuova finestra il programma lanciato è iniettabile nei test).

## Compiti – fase 2 (solo quando il PM te lo chiede)
6. Comandi `open_session(node, options, login?)` → `SessionId` (password di "chiedi all'avvio" in ingresso come
   `Secret`, DEC-016.3), `session_input`, `session_resize`, `session_close`, `open_in_terminal(node)`, sopra
   l'`AppState` di UI-WP2 e `portolano-ops::connect`; registrazione, permessi, bindings rigenerati.
7. Viste: tab SSH con il componente `Terminal` di UI-WP1 (stato unico, "Riprova", codice d'uscita), tab RDP (stato,
   durata che scorre, "Mostra log", "Disconnetti"), tab Windows App ("Aperta in Windows App"), con i `Tabs` di
   UI-WP1 e i testi dell'inventario §5.4. Il montaggio nella finestra principale è di UI-WP4: tu esporti le viste e
   una pagina della galleria o uno scenario e2e che le mostra con il trasporto finto.
8. e2e (`frontend/e2e/sessions*.spec.ts`) con il trasporto finto: apertura, output, stato connesso, uscita con
   codice, errore 255, "Riprova"; screenshot delle tre tab.

## Gate prima di consegnare (output reale nel report, a fine di ogni fase)
Rust: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
In `frontend/`: `npm run typecheck`, `npm run lint`, `npm run format:check`, `npm test`, `npm run build` e, in fase
2, `npm run bindings:check` e `npm run e2e`.
