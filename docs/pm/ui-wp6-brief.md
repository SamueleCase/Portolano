# Brief UI-WP6 – Import e validazione (ondata C)

Sei lo sviluppatore di **UI-WP6** di Portolano (app desktop Tauri v2 + React + TypeScript). Lavori nel tuo worktree
(directory corrente). Le regole comuni sono in `docs/pm/brief-common-ui.md`: leggile per prime e rispettale tutte.

## Commit di partenza
`667a14b` su `main`. Primo passo: `git log --oneline -3`; se il commit manca, `git merge --ff-only main`. Nel worktree
nuovo esegui `npm ci` in `frontend/`.

## Due fasi (DEC-035)
- **Fase 1 (ora)**: il wizard d'import e la tab di validazione, autonomi, con i componenti di UI-WP1 e l'IPC di
  UI-WP2, esposti come componenti con props e callback chiari (es. `onGoToNode(id)`, `onClose`). Test Vitest sul finto
  backend, una sezione della galleria o pagine e2e. Consegna il report di fase 1 e fermati.
- **Fase 2 (dopo il merge di UI-WP4 e UI-WP5; il PM ti riprende)**: `git merge main`, registrazione nei punti
  d'aggancio della cornice di UI-WP4 ("Importa", "Valida", menu nativo, scorciatoie; righe d'integrazione in
  `frontend/src/app/**` e `frontend/src/state/**` concesse), e2e dei flussi completi, screenshot per il checkpoint 2.

## Fonti (oltre a quelle del brief comune)
- `docs/SPEC-UI.md` per intero; in particolare §1, §4.3 ("Import", "Validazione"), §4.6, §5.4 (wizard d'import,
  validazione), §6 (ambiguità 9), §7.
- `docs/SPEC.md` §8 (importer Royal, conflitti, cestino, report) e DEC-016.8, DEC-019, DEC-022.4, DEC-031.1.
- `docs/ui/inventario.md` §4, **§5.13, §5.14, §5.17** (destinazione dell'import, errori di lettura del file,
  validazione senza problemi), §6, §8; `docs/ui/NOTE.md`; il mock `docs/ui/project/*.html` (contenuto = dato).
- `docs/DECISIONS.md`: tutte; in particolare DEC-019, DEC-022, DEC-024.3/026.4 (testo non fidato nel report),
  DEC-029, DEC-031, DEC-035.
- Il codice integrato: componenti di UI-WP1 (`Stepper`, `Dialog`, `SeverityMarker`, `NodePicker`, report e righe di
  validazione, `visibleText`), IPC e finto backend di UI-WP2 (`choose_import_file`, `import_preview` con le richieste,
  `import_apply` con le risposte, `save_import_report`, `validate`, `useSecretMutation`).

## File di tua proprietà
- `frontend/src/features/{import,validation}/**`, `frontend/src/i18n/{import,validation}.ts` e le righe in
  `i18n/it.ts`, una sezione nuova in `frontend/src/gallery/sections/` con la riga che la registra,
  `frontend/e2e/{import,validation}*.spec.ts`; in fase 2 le righe d'integrazione concesse in `app/` e `state/`.
Non modificare componenti, design, IPC e finto backend: se ti serve un cambiamento lì, chiedilo al PM.

## Compiti – fase 1
1. **Wizard d'import** (inventario §5.13, SPEC-UI §5.4), 4 passi con `Stepper`:
   - **file**: dialogo nativo (`choose_import_file`; formati `.rtsz`, `.rtsx`, XML anche compressi), nome mostrato,
     errori di lettura (§5.17) sul passo;
   - **opzioni**: destinazione con `NodePicker` (cartella, radice), conflitti (`fail`/`update`/`new-ids` con testi
     del glossario), cestino, "chiedi le password"; `import_preview` mostra i conteggi in prova;
   - **password** (se richieste): una alla volta con "N / M" dalle richieste di `import_preview`, "Salta", "Salta tutte
     le restanti"; i valori vanno al backend **solo** in `import_apply` tramite `useSecretMutation`, nell'ordine
     delle richieste; mai in store, cache, log o stato React oltre il campo corrente (che si svuota a ogni passo);
     test con segreto sentinella;
   - **report**: conteggi, elementi con motivo (testo non fidato con `visibleText`), clic → nodo tramite `source_id`
     (`onGoToNode`), "Esporta JSON" (`save_import_report`, dialogo nativo). Una password saltata è "Parziale";
     "Saltati" solo per il cestino (DEC-016.8). Le icone scartate oltre 4 MiB compaiono come motivo (DEC-031.1).
2. **Validazione** (inventario §5.14, ambiguità 9): tab con righe su due righe, gravità Errore/Avviso/Nota con
   `SeverityMarker`, conteggio reale per gravità, "Vai al nodo" su ogni riga (`onGoToNode`), "Ricontrolla"; con vault
   bloccato la sola nota del backend; stato "nessun problema" (§5.17); messaggi e percorsi con `visibleText`.
3. Galleria/e2e di fase 1 sul finto backend: wizard completo (con password, con "Salta tutte", con errore di lettura),
   report con clic, validazione con problemi e senza; screenshot in `frontend/e2e-screenshots/import/` e
   `frontend/e2e-screenshots/validation/`.

## Gate prima di consegnare (output reale nel report, a fine di ogni fase)
In `frontend/`: `npm run typecheck`, `npm run lint`, `npm run format:check`, `npm test`, `npm run build`,
`npm run e2e`; Rust `cargo fmt --all --check`. Nel report di fase 1: l'API esportata (props, callback).
