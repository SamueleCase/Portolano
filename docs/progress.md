# Stato di avanzamento

Stato finale della FASE 1 + FASE 2a (SPEC §0), scritto con WP7 e aggiornato dopo WP8. Il PM lo aggiorna alla
verifica finale. Serve anche a riprendere il lavoro se la sessione si interrompe.

## Pacchetti di lavoro (2026-11-11)

| Passo | Stato | Note |
|---|---|---|
| Prerequisiti | fatto | Rust 1.99.1, FreeRDP 3.33.1 (`sdl-freerdp`); `samples/` con 2 documenti, ignorato da git |
| WP0 – scheletro e contratto | **integrato** | merge `6e49dfd`; due giri di revisione indipendente; 30 test; gate verdi su `main` |
| WP1 – albero, percorsi, risoluzione | **integrato** | merge `93c03c2`; due giri di revisione; 157 test nel core; gate verdi su `main` |
| WP2 – store, migrazioni, icone, config | **integrato** | merge `400adf0`; tre giri di revisione; lock per operazione (DEC-016.1), symlink, gc sicuro, `MAX_DEPTH` (DEC-021) |
| WP3 – vault | **integrato** | merge `b569fb3`; due giri di revisione; lock inter-processo (DEC-017), zeroize di Argon2; core: 297 test + 1 ignorato (Keychain) |
| WP4 – connettori | **integrato** | merge `3df8c3a`; tre giri di revisione; 158 test; opzioni FreeRDP reali (DEC-020) |
| WP5 – importer Royal | **integrato** | merge `737c58c`; tre giri di revisione; 159 test; test sui campioni reali verde (DEC-019, DEC-021) |
| WP6 – `portolano-ops` + CLI | **integrato** | merge `c762281`; due giri di revisione per crate; 882 test nel workspace (DEC-018, DEC-022, DEC-023) |
| WP7 – README, smoke, stato finale | **integrato** | merge `b2a6a7f`; una revisione; `README.md`, `scripts/smoke.sh` (`SMOKE OK`), questo file; nessuna modifica al codice; aggiornamento finale dopo WP8 |
| WP8 – indurimento di sicurezza | **integrato** | merge `3273cbe`; due giri di revisione; modello di fiducia, opzioni ssh ammesse, programmi in config, caratteri di controllo, ciphertext Royal, memoria dei segreti, FreeRDP in primo piano (DEC-024, DEC-025, DEC-026) |
| WP9 – coerenza | **integrato** | merge `85c7c16`; caratteri di controllo rifiutati anche nella destinazione FreeRDP; messaggio d'errore di stdin non ripetuto |
| Verifica finale (PM) | **fatta** | 2026-11-11 su `85c7c16`: gate su `main` e su un clone pulito, smoke, campioni reali, `cargo install`, import dei campioni in prova, DEC-015 dal vivo |
| Verifiche dal vivo (utente) | da fare | Keychain reale, SSH e RDP su host reali, gateway RDP, nuova finestra (sotto) |

Verifica finale del PM (su `85c7c16`, sia nel repo sia in un clone pulito senza `samples/`): `cargo fmt --all
--check` pulito; `cargo clippy --workspace --all-targets -- -D warnings` pulito; `cargo test --workspace` 980 test
superati, 0 falliti, 2 ignorati (Keychain reale, campioni reali); nessun IP fuori da RFC 5737 in codice, script e
README; `scripts/smoke.sh` termina con `SMOKE OK`; `PORTOLANO_SAMPLES_DIR=samples cargo test --workspace --
--ignored samples` passa; `cargo install --path crates/portolano-cli` installa `~/.cargo/bin/portolano` 0.1.0 e
`--help` elenca i 13 comandi del §10; `import --dry-run` dei due campioni: documento 1 → 23 oggetti, 15 importati,
7 parziali (le 7 password cifrate), 1 saltato (cestino), 1 avviso (documento protetto da password); documento 2 → 8
oggetti, 8 importati; nessuna scrittura sul documento.

## Cosa funziona

Verificato da test automatici (unitari, di integrazione con `assert_cmd`) e dallo smoke test:

- **Documento**: formato JSON versionato (`schema_version` 1) con migrazioni, salvataggio atomico deterministico
  con `.bak`, lock per operazione e rilevamento delle modifiche esterne (DEC-016.1), profondità massima 50
  (DEC-021), icone personalizzate deduplicate con gc.
- **Albero**: percorsi con escape, `@id` e prefissi, fallback senza maiuscole, ambiguità con candidati;
  `move` con riordino e rifiuto dei cicli; risoluzione delle credenziali (`inherit`, `none`, `prompt`, inline,
  per id, per nome); validazione del core, dei protocolli e del vault (`missing_secret` come avviso).
- **Vault**: Keychain (servizio `dev.portolano`), vault su file Argon2id + XChaCha20-Poly1305 sicuro con più
  processi (DEC-017), vault in memoria per i test; master password da prompt o da `PORTOLANO_VAULT_PASSWORD`,
  mai passata ai processi figli (DEC-023).
- **CLI**: tutti i comandi del §10 (`init`, `tree`, `list`, `show`, `add`, `edit`, `move`, `delete`, `secret`,
  `connect`, `import`, `validate`, `config`), opzioni globali con precedenza flag > env > config > default, help e
  messaggi in italiano, codici d'uscita di DEC-022.7, `--json` per i comandi di lettura.
- **Piani di lancio**: SSH con password e passphrase via askpass, chiave su file, chiave nel vault via `ssh-add`,
  catene di jump con `$ref` e indirizzi, rilevamento dei cicli; RDP con FreeRDP (fit, fullscreen, fisso, scala,
  gateway, password via stdin) e Windows App (file `.rdp` senza password); nuova finestra per Terminal, iTerm,
  Ghostty e comando personalizzato. Nessun segreto in argv, env o `--print` (test con segreto sentinella).
- **Askpass**: server su socket Unix con token monouso, ogni segreto servito una volta, fallback su `/dev/tty`
  (test di integrazione con prompt simulati).
- **Import Royal**: XML, gzip e zip; albero da `ParentID`/`PositionNr`; credenziali nei modi 0–4; cestino;
  segreti chiesti con `--prompt-secrets` o segnalati come `Partial`; tipi non supportati conservati senza campi
  cifrati; conflitti `fail`/`update`/`new-ids`; report in tabella e JSON senza segreti né ciphertext.
- **Libreria `portolano-ops`** riusabile dall'app della FASE 2 (DEC-018.2, DEC-022).
- **Indurimento (WP8)**: il documento non è fidato come `~/.ssh/config` (DEC-024); `-o` di ssh limitato a un elenco
  chiuso di 23 opzioni, `StrictHostKeyChecking` solo `yes|ask|accept-new` (DEC-026.1); utente di destinazione
  secondo `valid_ruser` di OpenSSH, regola più severa per i jump (DEC-026.2); `[ssh] program` e
  `[rdp] freerdp_program` accettati solo come percorsi assoluti sicuri alla `safe_path()`, con errore limitato al
  protocollo (DEC-026.3); caratteri di controllo, DEL, C1 e controlli bidi resi visibili in ogni output, anche
  `--json`, e avviso `control_characters` di `validate` (DEC-024.3, DEC-026.4); ciphertext Royal riconosciuto
  anche dal valore (DEC-024.4); segreti da stdin e risposte askpass in buffer fissi azzerati, massimo 64 KiB
  (DEC-024.5); master password del vault su file di almeno 12 caratteri (DEC-024.7); FreeRDP in primo piano quando
  deve chiedere una password (DEC-025); help localizzato (DEC-026.5); test dello store sul segreto sentinella con
  `MemoryVault`.
- **Installazione**: `cargo install --path crates/portolano-cli` compila in release e installa `portolano`, il cui
  `--help` elenca i comandi del §10 (verificato da WP7 con una `--root` temporanea).

## Cosa non è verificato: da provare dal vivo

Richiedono la presenza dell'utente, host reali o il portachiavi reale; nessuna è stata eseguita in sviluppo.

1. **Keychain reale** (DoD): `add cred --ask-password`, `secret status` → presente, `delete` → l'elemento sparisce
   dal portachiavi. C'è anche il test `#[ignore]`: `PORTOLANO_TEST_KEYCHAIN=1 cargo test -p portolano-core --
   --ignored keychain`. Su binari non firmati compare il prompt del portachiavi dopo ogni ricompilazione (§5.2).
2. ~~**DEC-015, prompt di `ssh-add`**~~: **verificato dal PM** il 2026-11-11 con un `ssh-agent` temporaneo e una
   chiave usa e getta: il prompt è `Enter passphrase for (stdin): ` e la chiave viene caricata. Resta da provare
   `connect` con una chiave nel vault su un host reale (punto 5).
3. **DEC-020, gateway RDP**: su un host reale con RD Gateway, riuso delle credenziali sul gateway quando
   coincidono con quelle della destinazione (`u:`/`d:` omessi) e `/from-stdin:force` insieme a `/gateway:`.
4. **DEC-025/DEC-026, FreeRDP che deve chiedere una password**: quando manca la password della destinazione o il
   gateway ha una credenziale diversa, FreeRDP ora resta in primo piano nel terminale (warning nel piano). Va
   verificato come la chiede davvero: sul terminale, in una finestra grafica, oppure (password della destinazione
   su stdin e gateway con credenziale diversa) leggendo anche quella del gateway dallo stdin già chiuso.
5. **SSH su host reale** (DoD): `connect` di una connessione con password salvata entra senza prompt; con jump
   host che hanno a loro volta una password; chiave nel vault con passphrase.
6. **RDP su host reale** (DoD): con FreeRDP installato, `connect` di una connessione con credenziale salvata non
   chiede la password; Windows App apre il `.rdp` generato.
7. **Nuova finestra**: `connect --new-window` con Terminal, iTerm e Ghostty (lancia `osascript`/`open`).

## Ipotesi [ipo] ancora aperte dell'importer Royal

Da `royal-format.md` e DEC-019. Le fixture sintetiche le coprono, ma i nomi e i valori dei campi non sono
verificati su documenti reali: i nodi ricostruiti da questi campi risultano `Partial` ("campi Royal non
verificati") e conservano l'originale in `extra`. Per chiuderle serve il documento di test descritto in
`royal-format.md` §9, creato con Royal TSX con dati finti.

- **Contenitore**: `.rtsx` come XML in chiaro e `.rtsz` compresso nelle versioni storiche; gzip e zip sono gestiti
  ma mai visti in un file reale (i campioni sono XML con BOM).
- **Date**: `MM/dd/yyyy HH:mm:ss.fff` interpretate come UTC.
- **Credenziali**: `CredentialMode` 1 = chiedi all'avvio; `CredentialMode` assente = `Inherit` (DEC-006);
  `CredentialId` verso altri documenti (resta `ById` con stato `Partial`).
- **Chiavi**: `PrivateKeyMode` ≠ 0 con `PrivateKeyContent` = chiave incorporata; `Passphrase`;
  `CredentialKeyMode=0` sulla connessione SSH.
- **Secure Gateway e RD Gateway**: `SecureGatewayID`, `RDSGatewayID`, `GatewayHostName`, `GatewayUsageMethod`
  (1/2 = usato, 0/4 = non usato); `RoyalRDSGateway` diventa `Unsupported` e host/porta sono copiati nelle
  connessioni, con credenziale del gateway `Inherit`.
- **Campi personalizzati**: `CustomField1..10` e `CustomProtectedField1..10`; struttura di `CustomProperties`.
- **Icone**: `CustomImageName` come percorso nella libreria di Royal, `CustomImage` come immagine in base64,
  `CustomImageList`.
- **Note**: `Notes` in HTML convertito in testo.
- **Altri tipi** (VNC, Web, File Transfer, Dynamic Folder, …): nomi dei tipi ipotizzati; in ogni caso un tipo
  sconosciuto diventa `Unsupported`.
- **Riconoscimento dei campi cifrati** (DEC-019.3, DEC-024.4): per nome e, per i nomi non previsti, dal valore
  (base64 che decodifica ad almeno 32 byte, multiplo di 16, escluse note e immagini). È un'euristica: su documenti
  reali potrebbe scartare qualche campo non cifrato (il nodo risulta `Partial`); nelle fixture sono scartati e
  segnalati anche `Settings`/`CustomProperties` con password annidate.

## Definition of Done (SPEC §13)

- [x] Tutti i quality gate passano su un checkout pulito (clone di `main` a `85c7c16`).
- [x] `scripts/smoke.sh` termina con `SMOKE OK`.
- [x] `cargo install --path crates/portolano-cli` installa `portolano`, e `portolano --help` elenca i comandi del
      §10.
- [x] Il test di conteggio sui campioni reali passa (`PORTOLANO_SAMPLES_DIR=samples cargo test --workspace --
      --ignored samples`).
- [x] L'import dei due documenti in `samples/` riesce, con report: le 7 password cifrate risultano `Partial` e i
      conteggi coincidono (23 = 15 + 7 + 1; 8 = 8).
- [ ] Con il Keychain reale: `add cred --ask-password`, `secret status` → presente, `delete` → la voce sparisce:
      **da verificare dall'utente**.
- [ ] `connect --print` di SSH e RDP mostra piani corretti (fatto: smoke test e test dei piani); con FreeRDP,
      `connect` RDP con credenziale salvata non chiede la password: **verifica manuale dell'utente**.
- [ ] `connect` di una SSH con password salvata entra senza prompt: **verifica manuale dell'utente**.
- [x] Il README contiene requisiti, build, posizione dei file, uso di ogni comando, sicurezza (con il modello di
      fiducia di DEC-024), limiti noti.
- [x] `docs/progress.md` contiene un riepilogo di cosa funziona, cosa no e le ipotesi `[ipo]` ancora aperte.

## Come riprendere dopo un'interruzione
1. `git log --oneline` su `main` e `git branch` per vedere i branch dei WP non ancora integrati.
2. I worktree dei sottoagenti sono in `.claude/worktrees/`: un `git status` dentro ciascuno mostra il lavoro non
   committato.
3. Le decisioni prese sono in `docs/DECISIONS.md`; il processo è in `docs/PM-PROMPT.md` (con la modifica DEC-007:
   il PM procede in autonomia).
4. Per il WP in revisione: rieseguire i quality gate (SPEC §11.2) e `scripts/smoke.sh` sul suo branch e ripetere
   la revisione indipendente prima del merge.

## Seguiti
- Verifiche dal vivo elencate sopra (DEC-015, DEC-020, DEC-025/DEC-026, Keychain, host reali).
- Valutare lo spostamento di `secret_input` (lettura dei segreti in buffer fissi, oggi in `portolano-connectors`
  e usato da askpass e CLI) nel core, dove l'app della FASE 2 lo troverebbe insieme a `Secret`.
- Chiusi: test di WP2 sul segreto sentinella con `MemoryVault` (fatto in WP8); help della CLI in italiano con
  "Valori ammessi" e "Opzioni globali" separate (DEC-026.5).

## Lavori rimandati
- RDP integrato nell'app con IronRDP (DEC-013): dopo la UI della FASE 2.
- Fuori scope (SPEC §14): UI Tauri, terminale integrato, Wake-on-LAN (si conserva solo `wol_mac`), VNC/Web/SFTP,
  sincronizzazione o condivisione dei documenti, decrittaggio dei segreti Royal, Windows come piattaforma di test.

### Limiti accettati (documentati nel README)
- `init --force` non tocca il vault: i segreti del vecchio documento restano (DEC-022.10).
- `add ssh --key` scrive `IdentityFile`/`IdentitiesOnly` in `settings.options` (DEC-022.2).
- `import --verbose` non offerto (DEC-022.9).
- `secret set/clear` non raggiungono le credenziali inline dentro i `settings` (es. gateway RDP importato) e
  l'`update` dell'import non ne conserva i segreti (DEC-022).
- `config show` mostra le sezioni dei protocolli solo se presenti nel file (DEC-022).
- Un nodo chiamato come un riferimento per id è raggiungibile solo per id (DEC-014.3).
- I log dei client e i `.rdp` si cancellano al `connect` successivo dopo 10 minuti.
- Utenti SSH con spazi: ammessi (DEC-026.2), ma l'askpass non serve la password perché le regex del prompt del §7.2
  escludono gli spazi nell'utente; la chiede `ssh`.
- `safe_path` controlla il file e le directory del percorso scritto e di quello risolto, ma non le directory dei
  symlink intermedi di una catena di link.
- Le risposte askpass che contengono caratteri da escape JSON lasciano una copia non azzerata nel buffer di
  `serde_json`.
- Le opzioni ssh sugli algoritmi (`Ciphers`, `MACs`, `KexAlgorithms`, `PubkeyAcceptedAlgorithms`) possono
  abilitare algoritmi deboli (DEC-026.1).
- La master password presa da `PORTOLANO_VAULT_PASSWORD` resta nel blocco dell'ambiente del processo (DEC-024.5);
  `config.toml` e i processi dello stesso utente sono dentro il perimetro di fiducia (DEC-024).
- Help: restano in inglese `[OPTIONS]` e `[env: …]` (limite di clap, DEC-026.5).

### Sicurezza per la UI (FASE 2)
- Le icone SVG importate si mostrano solo come `<img>`, mai inserite nel DOM (XSS memorizzato, DEC-024).

### Esigenze della UI (FASE 2) aggiungibili senza rompere il core
Dall'analisi del mock in `docs/ui/` (vedi DEC-016 per quelle già decise):
- traccia dell'ereditarietà della credenziale (`resolve_trace`: da quale cartella arriva);
- conteggio degli utilizzi di una credenziale inclusi gli ereditati (con `resolve_credential` su `walk`);
- anteprima dell'import (conteggi e numero di password richieste) e prompter asincrono con totale ("3 / 12",
  "Salta tutte");
- dal report d'import al nodo (via `source_id`);
- motivi di `Partial` per campi Royal specifici, se richiesti;
- nuovi controlli di validazione (credenziale senza password, inline duplicate confrontando solo utente e dominio);
- icone della libreria Royal mappate sul set builtin e `IconStore::list()`;
- riepilogo host/porta dei nodi non supportati estratto dall'importer;
- preferenze di aspetto della UI in un file separato da `config.toml` (non come sezione di primo livello, DEC-011);
- annullamento di un'eliminazione con cancellazione differita dei segreti;
- `self_exe` per askpass e nuova finestra = binario CLI incluso nell'app (sidecar), non l'eseguibile Tauri;
- stato bloccato/sbloccato del vault gestito dalla UI;
- operazione "Duplica" che rigenera id e `SecretRef` (DEC-016.4);
- estensione del file del documento (`.portolano` nel mock, `portolano.json` nella SPEC) da decidere per
  l'associazione dei file;
- "utente proposto" per la modalità "chiedi all'avvio";
- segreti dell'editor in staging: tenuti in memoria come `Secret` fino al Salva, poi `vault.set` → `save` con
  rollback; una sostituzione usa un nuovo `SecretRef` e il vecchio si cancella dopo il salvataggio; "Cancella"
  differito al Salva (evita segreti orfani nel Keychain dopo Annulla);
- salvataggio dell'editor dopo `ModifiedExternally`: modifiche tenute come patch e riapplicate dopo il reload;
- "Ricontrolla" con il vault bloccato: una sola nota invece di un `missing_secret` per ogni password;
- `portolano-ops` per Tauri: `OpsError` serializzabile e input (`ConflictPolicy`, `DeleteOptions`, `Placement`,
  `TypeFilter`, `CredentialInput`) con `Deserialize`;
- richiesta del dominio in "chiedi all'avvio" decisa dal protocollo (oggi `ask_domain` è cablato su `rdp` in ops):
  futuro metodo di `Protocol`.

## FASE 2 – app desktop (avviata il 2026-11-12)

Specifica: `docs/SPEC-UI.md`; processo: `docs/PM-PROMPT-UI.md`; stack e mock: DEC-027. Materiale del PM per i
sottoagenti: `docs/pm/brief-common-ui.md` (regole comuni da allegare a ogni brief) e `docs/pm/review-template.md`
(modello del revisore).

| Passo | Stato | Note |
|---|---|---|
| Inventario del mock | fatto | `docs/ui/inventario.md` |
| Ricognizione tecnica | fatta | `docs/ui/ricerca-tauri.md` |
| UI-WP0 – prototipo e scheletro | **integrato** | merge `3c710d1`; due giri di revisione indipendente; punti incerti chiusi (DEC-028); gate su `main`: Rust 1035 test superati e 2 ignorati, frontend (typecheck, lint, format, Vitest, build, bindings) verdi, smoke `SMOKE OK`, e2e verdi (1 timeout di `page.screenshot` su 5 esecuzioni, affidato a UI-WP1) |
| UI-WP1…WP3 (ondata B) | in corso | brief in `docs/pm/ui-wp{1,2,3}-brief.md`; proprietà dei file e UI-WP3 in due fasi (DEC-029, DEC-030); checkpoint visivo con l'utente sulla galleria dei componenti |
| ↳ UI-WP1 design system | **integrato** | merge `53166dc`; due giri di revisione più i minori; gate su `main`: Rust 1035 + 2 ignorati, Vitest 336, e2e 10/10, smoke OK; vetro 29 disegni/s, p95 18 ms, fermo a pagina nascosta (DEC-032) |
| ↳ UI-WP2 comandi | correzioni fatte, merge di `main` in corso | branch `worktree-agent-a890726d4e55c4c46`; due revisioni indipendenti (Rust e frontend, DEC-031); poi secondo giro di revisione e merge |
| ↳ UI-WP3 sessioni | fase 1 fatta e revisionata, correzioni fatte | branch `worktree-agent-aefbdbbf96d392569` (ultimo `427d3bc`); fase 2 dopo il merge di WP1 e WP2, poi revisione completa |

Ripresa dell'ondata B: per ogni branch `git -C .claude/worktrees/<agent> log --oneline d49eb52..HEAD` e `status`; un
sottoagente interrotto si riprende con SendMessage se la sessione è la stessa, altrimenti con un nuovo sottoagente nel
worktree esistente a cui si passano brief e stato. Verifiche dal vivo aggiunte: codice d'uscita di FreeRDP 3 quando
l'utente chiude la finestra (se non è 0, la tab RDP mostrerebbe "non riuscita"). Limite (DEC-030.3): FreeRDP che deve
chiedere una password con quella della destinazione già su stdin non si apre in una tab: si usa "Apri in terminale
esterno".
| UI-WP4…WP6 (ondata C) | da fare | checkpoint visivo con l'utente sulle schermate |
| UI-WP7 + revisione di sicurezza (ondata D) | da fare | |
