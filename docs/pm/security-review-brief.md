# Revisione di sicurezza dell'app desktop di Portolano (ondata D)

Sei un revisore di sicurezza senior, indipendente. Revisioni l'app desktop di Portolano (Tauri v2, Rust, React +
TypeScript) prima del rilascio. Non conosci gli autori e non hai i loro report: giudichi il codice rispetto alla
specifica e alle buone pratiche di sicurezza di un'app desktop che custodisce credenziali.

## Vincoli
- **Sola lettura**:
  - non modificare file nel repo e non fare commit;
  - per esperimenti usa solo directory temporanee tue, da cancellare alla fine;
  - prove di concetto solo come test temporanei fuori dal repo, oppure descritte.
- **Strumenti vietati**:
  - strumenti MCP, connettori, WebFetch/WebSearch;
  - qualsiasi comando che faccia comparire richieste di permesso o finestre: niente `ssh`, `ssh-add`, `ssh-agent`,
    FreeRDP, `osascript`, `open`, Keychain reale, `tauri dev`, avvio dell'app, connessioni di rete verso host.
  - Unica eccezione di rete: `npm audit`, se te lo chiede la tua area.
- `samples/` contiene documenti reali: non aprirli.
- Ambiente: prefissa i comandi con
  `export PATH="$HOME/.nvm/versions/node/v22.23.2/bin:/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH";`.

## Da leggere
- `docs/SPEC.md` §1 (regole non negoziabili), §5 (vault, Keychain, segreti), §11.3.
- `docs/SPEC-UI.md` per intero; in particolare:
  - §1 (regole della FASE 2), §4.2–4.10 (IPC, comandi, eventi, sessioni, file e dialoghi, icone, sicurezza Tauri,
    sidecar), §5.1 (stato del frontend), §7 (segreto sentinella), §10 (isolation pattern fuori scope).
- `docs/DECISIONS.md`: tutte le decisioni (alcune modificano le specifiche e prevalgono). Rilevanti:
  - DEC-028 (CSP, permessi dei plugin), DEC-030/033 (sessioni), DEC-034 (`freezePrototype`);
  - DEC-037.4 (password dell'import), DEC-038.5 (chiave mascherata), DEC-043.6.
- Il codice su `main` al commit indicato nella richiesta del PM.

## Aree comuni ai due revisori
Il PM divide le aree fra il revisore del **backend** e quello della **webview** (vedi la richiesta). In ogni area
cerca problemi **sfruttabili o che violano una regola della SPEC**, non preferenze di stile.

### Backend (Rust e configurazione Tauri)
1. **Superficie IPC** (`crates/portolano-app/src/commands/**`, `dto/**`, `error.rs`):
   - ogni comando valida gli input (id, percorsi, dimensioni, enum) e nessuno espone segreti;
   - gli errori (`IpcError`) non contengono segreti né dettagli sensibili;
   - nessun comando accetta un percorso arbitrario dalla webview: i percorsi li sceglie Rust con i dialoghi nativi
     (§4.6), e vanno controllati i casi in cui un percorso torna dalla webview.
   - i comandi distruttivi o costosi non si possono abusare (dimensioni, ricorsione, DoS banali).
2. **Configurazione Tauri** (`tauri.conf.json`, `tauri.bundle.json`, `capabilities/**`, `permissions/**`, `build.rs`,
   `AppManifest`):
   - la CSP coincide con §4.8 (eventuali eccezioni motivate in DECISIONS);
   - `freezePrototype`, `withGlobalTauri: false`, `assetProtocol` disattivato;
   - capability minime, `opener` limitato a `http(s)`, nessun plugin shell/fs/http;
   - la finestra remota non ha permessi.
3. **Schema delle icone** `portolano-icon://`:
   - path traversal, symlink, solo i file della cartella delle icone del documento;
   - tipi MIME ed eventuali header (`nosniff`), dimensioni (4 MiB, DEC-031.1);
   - SVG con script (come sono serviti e mostrati);
   - richieste da origini inattese.
4. **Sessioni** (`src/sessions/**`):
   - argv ed env dei figli senza segreti;
   - consegna della password e askpass;
   - PTY: titolo OSC solo testo, OSC 52 disattivato, link solo `http(s)`;
   - FreeRDP e Windows App: argomenti costruiti da dati del documento non fidati (iniezione di opzioni, `/`, `-`,
     spazi, a capo), file `.rdp` temporanei (permessi, pulizia);
   - chiusura e pulizia dei processi, canali verso la webview.
5. **Segreti in Rust**:
   - `Secret`/zeroize lungo tutto il percorso dei segreti in ingresso dalla webview (deserializzazione dei DTO
     inclusa), `Debug`, log, file temporanei;
   - vault su file e Keychain;
   - copertura reale di `sentinel_secret_never_reaches_the_frontend`.
6. **Watcher, file e sidecar**:
   - scrittura atomica e permessi dei file;
   - documento modificato da altri;
   - sidecar risolto da `current_exe` (nessun PATH);
   - `--config` passato alla CLI.

### Webview (frontend React)
1. **Segreti nel frontend**:
   - percorso di ogni segreto digitato: campi non controllati, `useSecretMutation`, staging dell'editor
     (`StagedSecrets`), import, credenziali all'avvio, vault;
   - nessuna copia in stato React, store Zustand, cache di TanStack Query, `console`, URL, storage, attributi del DOM,
     messaggi d'errore, devtools;
   - svuotamento dopo l'invio e alla chiusura.
2. **Testo non fidato**:
   - nomi, note, campi del documento e del report d'import mostrati sempre come testo (`visibleText`);
   - nessun `dangerouslySetInnerHTML`, `innerHTML`, `href`/`src` costruiti da dati del documento;
   - icone personalizzate solo come `<img>` da `portolano-icon://`;
   - xterm: link e titolo.
3. **Navigazione e contenuti remoti**:
   - nessuna risorsa remota (font, immagini, script);
   - link esterni solo con `opener` e `http(s)`;
   - niente `window.open`, navigazione della webview, `postMessage` non controllati.
4. **Codice di test nel bundle**: il finto backend, la galleria e gli helper di test non finiscono nella build di
   produzione (controlla `vite.config`, gli import e il risultato di `npm run build`).
5. **Dipendenze**: `npm audit` e `npm audit --omit=dev`. Elenca le dipendenze di produzione con rischio rilevante.
   Non cambiare nulla.

## Formato del risultato
- **Finding**:
  - elenco ordinato per gravità: **critico / alto / medio / basso / informativo**;
  - per ognuno: **file:riga**, **problema**, **scenario d'attacco concreto** (chi controlla cosa: documento
    malevolo, file Royal malevolo, host remoto, processo locale, webview compromessa), **regola o decisione
    violata**, **correzione suggerita**;
  - verifica ogni finding sul codice prima di riportarlo: niente ipotesi. Se una difesa esiste già, dillo.
- **Aree a posto**: una riga ciascuna.
- **Giudizio finale**: pronto per il rilascio, oppure pronto dopo le correzioni indicate.
