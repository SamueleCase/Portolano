# Brief UI-WP7 – Bundle e chiusura (ondata D)

Sei lo sviluppatore di **UI-WP7** di Portolano (app desktop Tauri v2 + React + TypeScript). Lavori nel tuo worktree
(directory corrente). Le regole comuni sono in `docs/pm/brief-common-ui.md`: leggile per prime e rispettale tutte.

## Commit di partenza
L'ultimo commit di `main` indicato nella richiesta del PM. Primo passo: `git log --oneline -3`; se il commit manca,
`git merge --ff-only main`. Nel worktree nuovo esegui `npm ci` in `frontend/`.

## Fonti (oltre a quelle del brief comune)
- `docs/SPEC-UI.md` per intero; in particolare §1, §4.8 (sicurezza Tauri), §4.10 (sidecar e build), §5.5
  (tastiera), §7 (test, segreto sentinella), §9 (Definition of Done), §10 (fuori scope).
- `docs/DECISIONS.md`: tutte; in particolare DEC-028 (punti incerti, bundle), DEC-032.4 e DEC-036.1 (Tab in
  WebKit), DEC-035 (punti d'aggancio), DEC-037.4, DEC-043.9, DEC-044.3, **DEC-045** (piano di questa ondata).
- `docs/progress.md` (stato, limiti annotati delle ondate B e C), `README.md` (struttura e tono delle sezioni della CLI).
- Il codice integrato su `main`, in particolare `frontend/scripts/bundle.sh`, `crates/portolano-app/tauri*.json`,
  `frontend/e2e/**`, `frontend/src/test/fakeBackend*.ts`, `crates/portolano-app/src/tests.rs`
  (`sentinel_secret_never_reaches_the_frontend`).

## File di tua proprietà
- `README.md`, `frontend/scripts/**`, `.gitignore`.
- `frontend/e2e/**`, i soli spec nuovi o le pagine d'aiuto (`*Page.ts`, `fixtures.ts`).
- Per la pulizia di DEC-045.3:
  - il modulo comune nuovo dell'helper dei percorsi;
  - le righe che lo usano in `components/NodePicker.tsx`, `features/tree/treeModel.ts` e `features/dialogs/treeIndex.ts`;
  - `features/import/useImportWizard.ts` e `i18n/import.ts`, con i loro test.
- Eccezioni già concesse:
  - `frontend/src/test/fakeBackend*.ts`, solo per registrare ciò che serve all'e2e con sentinella (es. un registro
    delle risposte e degli eventi), senza cambiare il comportamento esistente;
  - `crates/portolano-app/tauri*.json` e `frontend/scripts/bundle.sh`, solo se il bundle non rispetta SPEC-UI §4.10.
- Più avanti: le correzioni dei finding della revisione di sicurezza, con le eccezioni che il PM ti indicherà.

`docs/progress.md` e `docs/DECISIONS.md` li aggiorna il PM.

## Compiti
1. **Bundle** (SPEC-UI §4.10, DoD §9). Esegui `npm run bundle` e verifica, senza aprire l'app:
   - `Portolano.app` e il `.dmg`;
   - il binario `Contents/MacOS/portolano-app` e il sidecar `Contents/MacOS/portolano` (CLI in release), che risponde
     a `--version` lanciato dal bundle;
   - la firma ad hoc con runtime rinforzato (`codesign -dv --verbose=2`, `codesign --verify --deep --strict`);
   - le dimensioni.

   Se manca qualcosa, correggi negli script o nella configurazione indicata. Aggiungi uno script di controllo
   `frontend/scripts/check-bundle.sh` (eseguito alla fine di `npm run bundle` o a parte, a tua scelta motivata) che
   fallisce se una di queste condizioni non vale. **Non** eseguire `open`, non lanciare l'app e non toccare il
   Keychain.
2. **README** (DoD §9): la sezione dell'app.
   - installazione dal bundle (`npm run bundle`, dove si trova `Portolano.app`, primo avvio con la firma ad hoc:
     tasto destro → Apri);
   - avvio in sviluppo (`npm run tauri dev`);
   - uso: primo avvio, albero e ricerca, editor e segreti in staging, sessioni SSH nel terminale integrato e RDP,
     import, validazione, preferenze, scorciatoie del menu nativo;
   - sicurezza: il frontend non riceve segreti, CSP e capability, nessun contenuto remoto, vault e Keychain;
   - limiti noti, presi da `progress.md`, DECISIONS e SPEC-UI §10;
   - test del frontend (gate, e2e, screenshot);
   - struttura del progetto con `crates/portolano-app` e `frontend/`.

   Aggiorna la riga che dice l'interfaccia grafica "prevista in una fase successiva". Stesso tono e stile delle
   sezioni esistenti, in italiano, senza host o IP reali.
3. **E2e mancanti** (DEC-045.4):
   - **segreto sentinella**: uno spec nuovo che attraversa tutti i flussi che ricevono segreti (password e chiave
     nell'editor in staging fino a Salva, dialogo della password dal dettaglio, credenziali all'avvio, sblocco e
     creazione del vault, password dell'import). Verifica che il segreto non compaia mai:
     - in nessuna risposta IPC né evento del finto backend (registro completo);
     - nel DOM dopo l'invio;
     - in `localStorage`/`sessionStorage`;
     - nella console;
     - nella cache di TanStack Query.

     Un controllo in negativo deve poter fallire: prova che il registro vede davvero le risposte (es. un valore non
     segreto noto compare).
   - **tastiera nella finestra principale**: con Option+Tab (come `gallery-keyboard.spec.ts`) si raggiungono:
     - i controlli della barra, la ricerca, i filtri, l'albero (una fermata), le azioni del dettaglio e il piè;
     - il focus è visibile a ogni fermata;
     - nei dialoghi e nel pannello dell'editor il focus resta dentro e torna al controllo d'origine alla chiusura.
   - **copertura della DoD**: confronta l'elenco dei flussi di §9 con gli spec esistenti e scrivi nel report la
     tabella flusso → spec e test. Aggiungi solo quello che manca davvero.
4. **Pulizie** (DEC-045.3):
   - helper dei percorsi leggibili in un modulo comune, usato anche da `NodePicker`, senza copie;
   - tolto il ramo `vaultUnavailable` del wizard e il suo testo, con test aggiornati;
   - `npm audit` e `npm audit --omit=dev`: riporta l'esito; non cambiare versioni senza chiedere (SPEC-UI §2.1).
5. **Gate completi** prima di consegnare: Rust e frontend del brief comune, più `npm run e2e` due volte con
   `PORTOLANO_E2E_PORT` della richiesta del PM, `bash scripts/smoke.sh` e `npm run bundle` con il controllo.

## Dopo la consegna
Il PM rilancia i gate, fa rivedere il WP e ti passa i finding della revisione di sicurezza da correggere (piccoli
commit, un finding per commit quando possibile, test che falliscono prima della correzione).
