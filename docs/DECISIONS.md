# Decisioni

Registro delle decisioni prese durante l'implementazione: domande poste all'utente e deviazioni o
chiarimenti rispetto a `SPEC.md`. Le voci più recenti stanno in fondo.

---

## 2026-11-10 – Prerequisiti e contratto di WP0

Risposta dell'utente: "Va bene b". Interpretata come approvazione di tutte le raccomandazioni proposte
(opzione B per i punti 2–4, "sì" per i punti 1 e 5).

### DEC-001 – Installazione della toolchain
- **Domanda**: rustup/cargo e FreeRDP non sono installati; si possono installare con brew?
- **Decisione**: sì. Installati `rustup` (Rust stable 1.99.1) e `freerdp` 3.33.1 (binario `sdl-freerdp`).
- **Motivo**: Rust è necessario per l'implementazione; FreeRDP serve a verificare `/from-stdin` (SPEC §7.3).
- **Nota**: il formula `rustup` di brew è keg-only: i comandi si eseguono con
  `PATH=/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH`. Il `~/.zshrc` dell'utente non è stato modificato.

### DEC-002 – `resolve_parent_path` e opzione `-p` (SPEC §4.2)
- **Domanda**: la firma `resolve_parent_path(&self, path)` non può creare cartelle e non ha il parametro
  `create_missing` citato nel testo.
- **Decisione**: due metodi su `Document`:
  - `resolve_parent_path(&self, path: &str) -> Result<(Option<Uuid>, String)>`: sola lettura;
  - `ensure_parent_path(&mut self, path: &str) -> Result<(Option<Uuid>, String)>`: crea le cartelle
    intermedie mancanti (usato da `add … -p`).
- **Motivo**: la risoluzione in sola lettura resta disponibile su un documento immutabile (`show`, `connect`, UI).

### DEC-003 – `AppConfig` e indipendenza del core dai protocolli (SPEC §1.5 e §9)
- **Domanda**: le sezioni `[ssh]` e `[rdp]` tipizzate nel core violerebbero la regola per cui un nuovo
  protocollo non modifica il core.
- **Decisione**: il core tipizza solo `default_document`, `[vault]` e `[terminal]`. Le altre sezioni di primo
  livello (`[ssh]`, `[rdp]`, …) sono conservate come tabelle TOML grezze (`BTreeMap<String, toml::Value>`) e
  ogni `Protocol` deserializza la propria (`SshConfig`, `RdpConfig` in `portolano-connectors`).
  Il formato di `config.toml` resta quello del §9.
- **Motivo**: rispetta il §1.5 senza cambiare nulla per l'utente. `config set` sulle sezioni dei protocolli è
  validato dal protocollo, non dal core.

### DEC-004 – Comando per `connect --new-window` (SPEC §7.1, §7.2)
- **Domanda**: `PlanContext` non contiene il percorso del documento né le opzioni di vault/config che il
  processo figlio deve ricevere.
- **Decisione**: `ConnectOptions` riceve il campo `relaunch_args: Vec<String>`. La CLI lo riempie con le
  opzioni globali effettivamente specificate (`--file`, `--vault`, `--vault-file`, `--config`); il piano lo
  antepone a `connect @<id>`.
- **Motivo**: la nuova finestra usa esattamente lo stesso documento, vault e config del comando che l'ha
  aperta, e `plan` resta puro. Questi valori non sono segreti.

### DEC-005 – Dettagli di WP0 non fissati dalla SPEC
- **Decisione**:
  - Rust **edition 2024**, versione minima = stable corrente (`rust-version` nel workspace);
  - i tipi del §3 senza `derive` esplicito nella SPEC hanno gli stessi derive di `Document`
    (`Debug, Clone, PartialEq, Serialize, Deserialize`), più `Default` dove la SPEC lo indica;
  - crate con `publish = false` e senza campo `license`.

### DEC-007 – Il PM procede in autonomia (modifica a `PM-PROMPT.md`)
- **Richiesta dell'utente**: "deve andare da solo".
- **Decisione**: il PM non chiede più conferme all'utente, compreso il checkpoint dell'ondata 0.
  - I dubbi che la SPEC non copre li decide il PM con l'opzione più aderente alla SPEC e a §1/§11.3, li registra
    qui e li riassume nell'aggiornamento di fine ondata.
  - Si interrompe l'utente solo per azioni che richiedono la sua presenza fisica (prompt del Keychain, test
    manuali su host reali) o per azioni irreversibili fuori dal repo.
  - In `.claude/settings.local.json` (non versionato) ci sono i permessi per lavorare senza prompt; `git push`
    resta vietato.

### DEC-008 – Validazione delle sezioni di config dei protocolli (decisa dal PM)
- **Dubbio** (emerso in WP0): DEC-003 affida al protocollo la validazione di `config set` su `[ssh]`/`[rdp]`, ma
  il trait `Protocol` non ha un metodo per farlo.
- **Decisione**: si aggiunge a `Protocol` il metodo obbligatorio
  `fn validate_config(&self, section: &toml::Value) -> Result<(), String>`; la sezione è quella con nome
  uguale a `id()`. La CLI chiama il protocollo, senza conoscere `SshConfig`/`RdpConfig`.
- **Motivo**: rispetta §1.5; un nuovo protocollo porta con sé la propria validazione.

### DEC-009 – Dove vive il client askpass (decisa dal PM)
- **Dubbio** (emerso in WP0): il protocollo askpass (§7.2) ha il server nei connettori e il client nella CLI
  (`commands/askpass.rs`), con il rischio di duplicare nomi delle variabili e formato dei messaggi.
- **Decisione**: entrambi i lati stanno in `portolano-connectors::ssh::askpass` (WP4), che espone la funzione
  client `run_client(prompt: &str) -> i32` (exit code) e le costanti delle variabili d'ambiente.
  `commands/askpass.rs` e `main.rs` (WP6) si limitano a rilevare `PORTOLANO_ASKPASS=1` prima di clap e a
  chiamarla.
- **Motivo**: un solo modulo possiede il formato del protocollo, e i test di integrazione sul socket del §7.4
  esercitano client e server reali nello stesso crate.

### DEC-010 – `resolve_credential` su nodi senza `CredentialSpec` (decisa dal PM)
- **Dubbio** (revisione di WP0): §4.3 dice "usa la spec del nodo", ma `Credential` e `Unsupported` non hanno una
  `CredentialSpec`.
- **Decisione**: `Connection` e `Folder` usano la propria `credential`. Un nodo `Credential` restituisce i propri
  campi con `origin = Credential { node: <se stesso> }`. Un nodo `Unsupported` restituisce
  `Error::NoCredential { id }` ("il nodo … non ha credenziali"), variante aggiunta da WP0.
- **Motivo**: `show --resolved` su una credenziale mostra in modo naturale i suoi dati; un nodo non supportato
  non ha nulla da risolvere e un errore esplicito è più chiaro di un risultato vuoto.

### DEC-011 – Chiavi sconosciute in `config.toml` (decisa dal PM)
- **Dubbio** (revisione di WP0): con DEC-003 le sezioni dei protocolli sono raccolte con `flatten`, quindi un
  refuso come `default_documnt = "…"` verrebbe accettato in silenzio.
- **Decisione**: `AppConfig::load` accetta come chiavi non riconosciute solo **tabelle** (le sezioni dei
  protocolli); qualsiasi altro valore di primo livello sconosciuto dà `Error::InvalidConfig` con il nome della
  chiave. La validazione del contenuto delle sezioni resta ai protocolli (DEC-008).
- **Motivo**: un refuso in una chiave scalare non passa inosservato, senza che il core conosca i protocolli.

### DEC-012 – Dipendenze aggiunte da WP0 su richiesta della revisione
- `flate2` e `zip` (per l'importer, §8.3.1) e `toml` nella CLI (per `config show/set`, DEC-003/008) sono
  dichiarate da WP0, perché i `Cargo.toml` sono di sua proprietà (§12). `rand` è rimosso: non serve,
  `getrandom` copre tutti gli usi.

## 2026-11-11

### DEC-013 – Connessioni integrate nell'app: rimandate
- **Domanda dell'utente**: la connessione non è integrata nell'app?
- **Decisione**: in FASE 1/2a si resta come da SPEC: SSH nel terminale (corrente o nuova finestra), RDP nella
  finestra di FreeRDP. Il terminale SSH integrato è previsto con la UI (FASE 2). L'**RDP integrato** (IronRDP)
  si farà **in seguito**, come lavoro separato, non ora.
- **Motivo**: scelta dell'utente; il core attuale (modello, vault, piani di lancio) resta la base anche per la
  versione integrata.

### DEC-014 – Dettagli di percorsi e riferimenti (domande di WP1, decise dal PM)
1. **Radice**: si aggiunge `pub fn resolve_location(&self, path: &str) -> Result<Option<Uuid>>`, dove `None` è
   la radice (coerente con `children(Option<Uuid>)` e `move_node(.., Option<Uuid>, ..)`). `resolve_path("/")`
   restituisce `InvalidPath` ("la radice non è un nodo"). La CLI usa `resolve_location` dove la radice è un
   valore valido (`tree /`, `list /`, `move X /`).
2. **`@id` dopo il primo segmento**: si cerca fra i **discendenti** (a qualsiasi profondità) dei candidati
   correnti; in prima posizione equivale a tutto il documento.
3. **Nomi che sembrano un id**: è un riferimento per id solo `@` + UUID o `@` + ≥ 6 cifre esadecimali; il resto è
   un nome. Un nodo chiamato esattamente come un riferimento per id è raggiungibile solo per id (limite accettato,
   nessun escape `\@`).
4. **`references_to` e `ByName`**: conta un `ByName` che **risolve** al target secondo il §4.3 (incluso il
   fallback case-insensitive), così `delete` non lascia riferimenti rotti senza avviso.
5. **`secret_refs_in` e i `settings`**: il core cerca anche nei `settings` gli oggetti nella forma di una propria
   `CredentialSpec::Inline` (`"mode": "inline"`) e ne raccoglie la `password`. È il formato di un tipo del core,
   come la convenzione `$ref`, quindi il core non conosce i protocolli; evita segreti orfani nel vault quando si
   cancella una connessione RDP con gateway inline (possibile da import).
6. **Forma di `path_of`**: con lo `/` iniziale (`/Clienti/ACME/web01`), senza il nome del documento.
- Inoltre approvato: escape sconosciuti, `\` finale e `/` finale danno `InvalidPath`; tutti i codici di
  `validate` del core hanno severità `Error`.

### DEC-015 – Prompt della passphrase di `ssh-add` (decisa dal PM)
- **Dubbio**: con la chiave letta da stdin (`ssh-add -t 300 -`, §7.2), OpenSSH chiede la passphrase con un prompt
  della forma `Enter passphrase for (stdin): ` (eventualmente seguito da ` (will confirm each use)`), che non
  corrisponde alla regex del §7.2 `^Enter passphrase for key '(?P<path>.+)': ?$` (quella è di `ssh`).
- **Decisione**: il server askpass riconosce anche la forma di `ssh-add`:
  `^Enter passphrase for (?P<path>[^']+?)(?: \(will confirm each use\))?: ?$`, che produce un
  `PromptMatcher::Passphrase { key_path }`. Per lo step `SshAddKey` la voce askpass usa `key_path = "(stdin)"`.
  Le due regex restano distinte e testate entrambe.
- **Verifica**: il testo esatto del prompt si conferma nella verifica finale con un `ssh-agent` temporaneo e una
  chiave di test usa e getta (mai l'agent dell'utente); se diverge, si aggiorna la regex.
- **Verificato dal vivo il 2026-11-11** (OpenSSH_10.3p1, agent temporaneo, chiave ed25519 usa e getta): il prompt è
  esattamente `Enter passphrase for (stdin): `, corrisponde alla regex di questa decisione e non a quella del §7.2;
  con l'askpass che risponde, la chiave viene caricata (`Identity added: (stdin)`, durata 5 minuti).
- **Motivo**: senza questa forma la passphrase delle chiavi nel vault non verrebbe mai servita dall'askpass.

### DEC-016 – Esigenze del mock UI che toccano il core (decise dal PM)
Dall'analisi del mock in `docs/ui/` (FASE 2) contro SPEC e contratto. Si decidono ora perché costerebbe caro
cambiarle dopo; le esigenze aggiungibili in FASE 2 senza rompere nulla sono in `progress.md`, le correzioni al solo
mock in `docs/ui/NOTE.md`.
1. **Lock del documento (WP2)**: niente più lock esclusivo per tutta la vita dello `Store`, che bloccherebbe la CLI
   (anche `connect --new-window`) mentre l'app è aperta. Il lock su `<file>.lock` si prende solo durante le
   operazioni: condiviso in `load`, esclusivo in `save`/`create`. Lo `Store` ricorda l'impronta (SHA-256) dei byte
   letti o scritti per ultimi; `save` ricontrolla l'impronta sotto lock esclusivo e, se il file è cambiato nel
   frattempo, fallisce con `Error::ModifiedExternally { path }` ("il documento … è stato modificato da un altro
   processo: ricaricalo"). Sostituisce il requisito "il lock impedisce un secondo `Store::open`".
2. **Lancio in due passi (WP4)**: `execute` si divide in una preparazione (avvia il server askpass, risolve i
   segreti, produce programma/argomenti/env del passo interattivo e un guard che al drop chiude socket e azzera i
   segreti) e un avvio. `execute` resta come composizione dei due. Serve al terminale integrato della FASE 2
   (DEC-013), che avvierà `ssh` in una propria PTY.
3. **Credenziale "chiedi all'avvio" (WP4, poi WP6)**: `ConnectOptions` riceve
   `session_credential: Option<SessionCredential { username, domain, password: Option<SecretRef> }>`. Con origine
   `Prompt`: se presente si usa come una `Inline`; se assente il piano non passa utente né password e lascia che
   sia il client a chiederli (warning nel piano). La CLI chiede utente e password su TTY e mette la password in un
   vault in memoria per quella sola sessione. `CredentialSpec::Prompt` resta senza campi (l'"utente proposto" del
   mock non si fa ora).
4. **Invariante dei segreti (WP1)**: un `SecretRef` appartiene a un solo punto del documento. `validate` lo
   controlla con il codice `duplicate_secret_ref` (Error). Evita che cancellare un nodo cancelli la password di un
   altro (es. dopo un futuro "Duplica").
5. **Eliminazione forzata di una credenziale referenziata (WP1, poi WP6)**: nuova funzione del core
   `detach_credential_references(&mut self, credential: Uuid) -> Vec<Uuid>`: ogni `CredentialSpec` (sul nodo o
   dentro i `settings`) che punta alla credenziale per id o per nome diventa `Prompt`; restituisce i nodi modificati
   e ne aggiorna `modified`. `delete --force` la usa, così le connessioni restano utilizzabili chiedendo le
   credenziali.
6. **Livello `Info` nella validazione (WP1)**: `Severity` riceve `Info`; il core segnala i nodi non supportati con
   `unsupported_node` (Info). L'exit 4 di `validate` resta solo per gli `Error`.
7. **Client RDP per connessione (WP4)**: `RdpSettings.client: Option<RdpClient>`; precedenza
   `--client` > impostazione della connessione > `config.rdp.client`.
8. **Restano come da SPEC** (si corregge il mock): `ByName` cerca in tutto il documento (§4.3); una password Royal
   saltata è `Partial`, `Skipped` vale solo per il cestino (§8.3); la condivisione di documenti in cartelle
   sincronizzate resta fuori scope (§14).

### DEC-017 – Vault su file con più processi (decisa dal PM)
- **Dubbio** (report di WP3): `FileVault` riscrive `vault.json` dalla propria copia in memoria; con due processi
  aperti (app e CLI, DEC-016.1) l'ultimo che scrive cancella i segreti aggiunti dall'altro.
- **Decisione**: `set` e `delete` fanno lettura-modifica-scrittura sotto lock esclusivo su `vault.json.lock`
  (`fd-lock`): rileggono e decifrano il file corrente, applicano la modifica, riscrivono in modo atomico. `get` e
  `contains` leggono il file corrente sotto lock condiviso (la chiave derivata resta in memoria, il salt è fisso
  per file). Nessuna copia dei segreti tenuta in memoria fra un'operazione e l'altra.
- Inoltre: i parametri KDF letti dal file hanno limiti massimi ragionevoli (oltre → `VaultError::Corrupt`), così
  un file manomesso non blocca il processo; abilitata la feature `zeroize` di `chacha20poly1305` e `argon2`; il
  modulo `vault/file.rs` è diviso in sottomoduli `vault/file/*.rs`.

### DEC-018 – Eliminazione e orchestrazione riusabile dall'app (decise dal PM, dal mock aggiornato)
1. **Regola di `delete`** (sostituisce la riga `delete` del §10.2 per la parte sui riferimenti): si contano i
   riferimenti **da fuori** del sottoalbero eliminato verso **qualunque** nodo del sottoalbero (credenziali, jump
   host, …), usando `references_to`. Senza `--force` l'eliminazione si rifiuta per ogni tipo di nodo, con l'elenco
   dei riferimenti. Con `--force`: per ogni credenziale del sottoalbero si chiama `detach_credential_references`
   (i nodi passano a `Prompt`, DEC-016.5); gli altri riferimenti (es. `$ref` dei jump) restano e `validate` li
   segnala come `dangling_ref`, così il core non conosce i protocolli. Restano invariati `-r`, `-y` e la
   cancellazione dei segreti del sottoalbero dopo il salvataggio.
2. **Orchestrazione in una libreria**: le operazioni che la SPEC assegna alla CLI e che l'app dovrà rifare
   (inserimento dell'import §8.4 con conflitti, ordine segreti → icone → documento e rollback; composizione di
   `validate` con protocolli e `missing_secret`; `delete` con riferimenti e segreti; `secret set/clear`; `add`/`edit`
   con i segreti) vanno in un nuovo crate **`portolano-ops`** (`crates/portolano-ops`, dipende da core, connectors,
   import). Funzioni pubbliche senza stampe né TTY: ricevono `&Store`, `&dyn Vault`, i registry e callback per
   prompt e conferme, e restituiscono risultati strutturati. La CLI (`portolano-cli`) dipende da `portolano-ops` e i
   suoi `commands/*.rs` sono adattatori sottili (parsing, I/O, output). Entrambi i crate sono di WP6.
- **Motivo**: l'app della FASE 2 esegue gli stessi flussi (import, validazione, eliminazione, segreti); senza una
  libreria Tauri dovrebbe duplicare la logica (SPEC §0: "il core va esposto come libreria pulita").

### DEC-019 – Scelte dell'importer Royal sui campi [ipo] (proposte da WP5, approvate dal PM)
1. Nomi di campo ipotizzati: `SecureGatewayID`, `RDSGatewayID`, `GatewayHostName`, `GatewayUsageMethod`,
   `CustomProtectedField1..10`, `PrivateKeyContent` (con `PrivateKeyMode` ≠ 0), `Passphrase`. Ogni dato ricostruito
   da questi campi lascia l'originale in `extra` e rende il nodo `Partial` ("campi Royal non verificati").
   `GatewayUsageMethod`: 1/2 → gateway usato; 0/4 → nessun gateway; altro o assente → nessun gateway e `Partial`.
2. `RoyalRDSGateway` diventa un nodo `Unsupported` (un `RdpGateway` non può riferirsi a un nodo); host e porta sono
   copiati nelle connessioni che lo usano, la credenziale del gateway resta `Inherit`.
3. I campi cifrati si riconoscono per nome (password, passphrase, secret, chiave incorporata, campi custom
   protetti) e non vengono mai conservati, nemmeno nel `raw` dei nodi `Unsupported` (D3). Un campo cifrato non
   gestito viene scartato e il nodo è `Partial`.
4. La password si chiede solo per il modo di credenziale attivo; `CredentialPassword` con un altro modo si scarta.
5. `DOMINIO\utente` si divide anche nelle credenziali inline (modo 2).
6. Fermano l'import (`InvalidInput`): `ID` mancante, non valido o duplicato; `RoyalDocument` assente o ripetuto.
   `ParentID` inesistente, non cartella o ciclico → nodo sotto la radice con warning.
7. Valori non validi (porta, colore, date, immagine) restano in `extra` e contano fra i campi non mappati; un'immagine
   non riconosciuta rende il nodo `Partial`. Con `CustomImage` e `CustomImageName` insieme vince l'immagine.
8. Nei test d'integrazione un `#![allow]` di crate per `unwrap`/`expect`/`panic`, motivato (le funzioni di supporto
   dei test non sono `#[test]`).
- **Campioni reali**: con `PORTOLANO_SAMPLES_DIR=samples` dalla radice del repo il test di conteggio passa; i 7
  `Partial` del documento con password sono le 7 password cifrate attese.

### DEC-020 – Opzioni reali di FreeRDP 3.33 (verificate da WP4 con `sdl-freerdp /help`)
- Il gateway si indica con `/gateway:g:<host>[:<port>],u:<user>,d:<domain>` (l'opzione `/gw:` del §7.3 non esiste);
  mai `p:`.
- Password da stdin con `/from-stdin:force` (la password è letta prima della connessione).
- Non esiste un'opzione esplicita per riusare le credenziali sul gateway: se la credenziale del gateway coincide con
  quella della destinazione si omettono `u:`/`d:` e si conta sul comportamento predefinito di FreeRDP; altrimenti
  si passano `u:`/`d:` con il warning "la password del gateway la chiederà FreeRDP".
- **Da verificare dal vivo** (verifica manuale dell'utente su un host reale con gateway): riuso delle credenziali sul
  gateway e `/from-stdin:force` insieme al gateway.
- Prima dell'host, `ssh` riceve `--`, così un host preso dal documento non può essere interpretato come opzione.

### DEC-021 – Profondità massima dell'albero (decisa dal PM)
- **Dubbio** (revisione di WP5): `serde_json` ha un limite di ricorsione di 128 livelli, cioè circa 63 cartelle
  annidate; un albero più profondo si salverebbe ma non si ricaricherebbe. Inoltre le funzioni ricorsive
  (importer, albero) vanno in stack overflow con annidamenti di migliaia di livelli. §4.1 assumeva < 1000.
- **Decisione**: profondità massima **50 livelli** di nodi (i figli della radice sono al livello 1), costante pubblica
  `MAX_DEPTH` nel modello del core. `insert`, `move_node` ed `ensure_parent_path` rifiutano un'operazione che la
  supererebbe con `Error::TooDeep { max }` ("l'albero supererebbe la profondità massima di {max} livelli"). Lo store
  verifica che un documento alla profondità massima, con impostazioni annidate, faccia andata e ritorno. L'importer
  aggancia alla radice, con warning e stato `Partial`, gli oggetti che supererebbero il limite.
- **Motivo**: un limite esplicito e documentato è più chiaro di un errore di parsing a sorpresa; 50 livelli sono ben
  oltre l'uso reale (nei campioni la profondità è 3).

### DEC-022 – API di `portolano-ops` e dettagli della CLI (piano di WP6, approvato dal PM)
- **API**: approvata come proposta da WP6. Principi: nessuna stampa/TTY/exit; le funzioni che modificano lavorano su
  una copia e sostituiscono il documento solo se tutto riesce; un unico percorso di salvataggio (segreti → icone →
  documento, rollback dei segreti se il salvataggio fallisce, cancellazione dei segreti non più usati dopo il
  salvataggio, con gli errori di pulizia riportati nell'esito); conferme e richieste all'utente come API in due fasi
  (`preview_delete` → `delete`, `login_request` → `plan_connection`); vault aperto al primo uso (`LazyVault`);
  `SessionVault` per DEC-016.3; `validate` con vault facoltativo (vault bloccato nella UI).
1. `secret set` su un segreto già referenziato **sovrascrive la voce sul posto** senza cambiare il documento (il
   documento può essere condiviso fra macchine con portachiavi diversi). Lo staging con riferimento nuovo resta
   un'esigenza dell'editor della UI in FASE 2.
2. `add ssh --key FILE`: la chiave va in `settings.options` come `IdentityFile=<path>` e `IdentitiesOnly=yes`
   (`CredentialSpec::Inline` non ha un campo per la chiave). Limite: `show --resolved` non la mostra come chiave della
   credenziale. Per una chiave riusabile si usa `add cred --key` più `--cred`.
3. `--jump`: se il valore risolve a un nodo diventa `$ref` (deve essere una connessione `ssh`); `NotFound` senza `/`
   né `@` iniziale → indirizzo; `NotFound` con `/` o `@` → errore; `Ambiguous` → errore.
4. `missing_secret` ha severità **Warning** (multi-macchina: il documento è coerente, manca la voce nel vault locale).
5. `add`, la rinomina di `edit` e `move` rifiutano un nome già usato da un fratello (`NameTaken`, confronto esatto);
   l'import resta libero.
6. `--vault-file`/`PORTOLANO_VAULT_FILE` senza `--vault`/`PORTOLANO_VAULT` implica il backend `file`; con
   `--vault keychain` esplicito è un errore d'uso (exit 2).
7. Codici d'uscita: 2 uso (clap, `--json` su un comando di scrittura); 3 non trovato/ambiguo; 4 validazione e input
   che renderebbe invalidi documento o config; 5 vault (anche segreti rimasti dopo un salvataggio riuscito, con
   avviso); 6 connessione prima del lancio; 1 il resto (inclusi riferimenti senza `--force`, cartella non vuota
   senza `-r`, `ModifiedExternally`, `DocumentMissing`); altrimenti il codice del client.
8. `relaunch_args` contiene anche i valori arrivati da variabili d'ambiente, con percorsi assoluti;
   `PORTOLANO_VAULT_PASSWORD` non si passa mai.
9. `import --verbose`: non offerto (l'importer conserva negli `items` solo i non `Imported`); limite annotato.
10. `init --force` sovrascrive il documento (resta il `.bak`) e non tocca il vault: i segreti del vecchio documento
    restano nel vault; da documentare nel README.
- Limiti annotati: `secret set/clear` non raggiunge le credenziali inline dentro i `settings` (es. gateway RDP
  importato) e l'`update` dell'import non le conserva; `config show` mostra le sezioni dei protocolli solo se
  presenti nel file.

### DEC-023 – Master password del vault mai nell'ambiente dei figli; CLI in italiano (decise dal PM)
1. **Problema** (revisione di WP6): `PORTOLANO_VAULT_PASSWORD`, letta dalla CLI, restava nell'ambiente ereditato da
   `ssh`, FreeRDP, `ssh-add` e terminale (§1.1 lo vieta). **Decisione**: il nome della variabile diventa una costante
   pubblica del core (`portolano_core::vault::MASTER_PASSWORD_ENV`), usata dalla CLI; `portolano-connectors` la toglie
   (`env_remove`) dall'ambiente di **ogni** processo figlio che lancia. Test con una sonda che fallisce se la variabile
   è presente. Eccezione di proprietà: WP6 modifica per questo `vault/mod.rs` del core e `exec.rs` dei connettori.
2. **Messaggi della CLI**: gli errori d'uso e l'help di clap vanno in italiano (§10.3): `try_parse`, traduzione dei
   tipi di errore principali mantenendo l'exit 2, intestazioni e descrizioni dei flag di aiuto in italiano.

### DEC-024 – Modello di fiducia e indurimento (dalla revisione di sicurezza, decise dal PM)
- **Modello di fiducia**: il documento può arrivare da altri (repo condiviso, import Royal) e **non** è fidato quanto
  `~/.ssh/config`. La configurazione (`config.toml`) e i processi dello stesso utente sono dentro il perimetro di
  fiducia, come per gli helper di credenziali di git: un processo dell'utente che riscrive la config può ottenere i
  segreti, e questo rischio residuo è accettato e documentato nel README.
1. **Opzioni ssh pericolose**: `SshSettings.options` rifiuta (in `validate_settings`, quindi in `add`/`edit`, e come
   errore in `validate` e in `connect` per documenti scritti a mano o importati) le opzioni che eseguono comandi o
   cambiano ambiente/agent: `ProxyCommand`, `ProxyJump` (si usa `jump_hosts`), `LocalCommand`, `PermitLocalCommand`,
   `KnownHostsCommand`, `PKCS11Provider`, `SecurityKeyProvider`, `ProxyUseFdpass`, `SendEnv`, `SetEnv`, `Include`,
   `Match`, `IdentityAgent`, `ControlPath`, `ControlMaster`, `RemoteCommand`, `ForwardAgent` (si usa
   `agent_forwarding`), confronto senza distinguere maiuscole. Chi ne ha bisogno le mette nel proprio `~/.ssh/config`.
2. **Programmi sostituiti in config**: `[ssh] program` e `[rdp] freerdp_program` devono essere percorsi assoluti a file
   regolari non scrivibili dal gruppo o da altri; altrimenti `connect` rifiuta con errore chiaro. Rischio residuo
   (stesso utente) documentato.
3. **Caratteri di controllo**: l'output della CLI (`tree`, `list`, `show`, `validate`, report d'import, `connect
   --print`) li rende visibili (es. `\u{1b}`) invece di emetterli; `validate` segnala con un Warning
   `control_characters` nomi, host e utenti che li contengono; host e utente di destinazione SSH rifiutano gli stessi
   caratteri vietati dei jump (DEC sul `-J`) e i caratteri di controllo. Requisito documentato: OpenSSH ≥ 8.4.
4. **Ciphertext Royal con nomi non previsti**: oltre al riconoscimento per nome, l'importer tratta come cifrato
   (scartato, nodo `Partial`) un valore base64 che decodifica a ≥ 32 byte con lunghezza multipla di 16, nei campi che
   non sono note né immagini (D3).
5. **Memoria**: le letture di segreti da stdin (`--password-stdin`, `--key-stdin`) e le risposte askpass usano buffer
   preallocati che si azzerano, con un limite di dimensione; la master password presa dall'ambiente resta nel blocco
   environ del processo (limite documentato).
6. `RawField`/`RawObject` dell'importer con `Debug` manuale che non mostra i valori.
7. Master password del vault su file: minimo 12 caratteri alla creazione.
- Nota per la FASE 2: le icone SVG importate vanno mostrate solo come `<img>`, mai inserite nel DOM.

### DEC-025 – FreeRDP che deve chiedere una password (dal report di WP7, decisa dal PM)
- **Problema**: senza `--wait` FreeRDP è staccato con stdin vuoto (o chiuso dopo la password), quindi non può chiedere
  all'utente la password mancante della destinazione o quella del gateway con credenziale diversa.
- **Decisione**: quando FreeRDP dovrà chiedere qualcosa (nessuna password per la destinazione, oppure gateway con
  credenziale diversa da quella della destinazione), il piano lo esegue in primo piano (`wait = true`, terminale
  ereditato) con un warning che lo spiega; negli altri casi resta staccato. Comportamento da confermare nella verifica
  manuale dell'utente.

### DEC-026 – Opzioni ssh ammesse invece che vietate, e rifiniture dell'indurimento (revisione di WP8, decise dal PM)
1. **Lista di opzioni ammesse** (sostituisce l'elenco di divieti di DEC-024.1): la revisione ha trovato alias e opzioni
   non elencate che eseguono programmi (`SmartcardDevice`, `XAuthLocation`) o aprono la rete dell'utente e
   silenziano il controllo della host key (`RemoteForward`, `DynamicForward`, `StrictHostKeyChecking=no`, …). Una lista
   di divieti insegue sempre nuove parole chiave: `SshSettings.options` accetta **solo** opzioni di un elenco chiuso
   che non eseguono comandi, non aprono inoltri o tunnel, non cambiano destinazione, utente, porta, agent, ambiente o
   file letti, e non indeboliscono la verifica della host key. Esempi: `IdentityFile` e `IdentitiesOnly` (DEC-022.2),
   `ConnectTimeout`, `ConnectionAttempts`, `ServerAliveCountMax`, `TCPKeepAlive`, `PreferredAuthentications`,
   `PubkeyAuthentication`, `PasswordAuthentication`, `KbdInteractiveAuthentication`, gli algoritmi (`Ciphers`,
   `MACs`, `KexAlgorithms`, `HostKeyAlgorithms`, `PubkeyAcceptedAlgorithms`), `Compression`, `LogLevel`,
   `RequestTTY`, `AddKeysToAgent`, `CheckHostIP`, `VisualHostKey`, `AddressFamily`; `StrictHostKeyChecking` solo con
   `yes`, `ask` o `accept-new`. L'elenco definitivo, con il motivo di ciascuna voce, sta nel codice. Tutto il resto va
   nel `~/.ssh/config` dell'utente.
2. **Utente di destinazione SSH**: allineato a OpenSSH (`valid_ruser`): ammessi lo spazio non seguito da `-` e il
   backslash non finale (es. `DOMINIO\utente` per server Windows); restano vietati i metacaratteri della shell e i
   caratteri di controllo. Gli utenti e gli host dei jump mantengono la regola più severa (`-J` è separato da
   virgole).
3. **Programmi in config** (DEC-024.2 rafforzata come `safe_path()` di OpenSSH): il file e tutte le directory del suo
   percorso (anche dopo i symlink) devono appartenere all'utente o a root e non essere scrivibili da gruppo o altri.
   Un `[ssh] program` non valido non impedisce le connessioni RDP e viceversa: l'errore riguarda solo il protocollo
   usato.
4. **Output**: anche l'output `--json` rende visibili DEL e i caratteri di controllo C1 (serde_json esegue l'escape
   solo di U+0000–U+001F); anche i controlli bidi (U+202A–U+202E, U+2066–U+2069, U+200E/F, U+061C) sono resi visibili.
5. **Help in italiano**: la localizzazione dell'help (intestazioni "Opzioni globali", "Valori ammessi",
   "predefinito") è un'estensione di DEC-023.2; restano in inglese `[OPTIONS]` e `[env: …]` (limite di clap).
6. Il prompt della nuova master password indica il minimo di 12 caratteri.
- **Da verificare dal vivo** (DEC-025): con password della destinazione su stdin e gateway con credenziale diversa,
  FreeRDP potrebbe leggere anche la password del gateway da stdin (chiuso) o chiederla con una finestra grafica.

### DEC-027 – Avvio della FASE 2 (UI): stack e mock (decisione dell'utente, 2026-11-11)
- **Stack confermato dall'utente**: app desktop **Tauri v2** (Rust) che usa i crate esistenti (`portolano-ops` in
  testa) + interfaccia **React + TypeScript**. Valutato Flutter e scartato: l'RDP integrato si farà con **IronRDP**
  (Rust, con client web WASM/canvas), che in Tauri si integra direttamente; con Flutter servirebbe un bridge
  Rust↔Dart per tutto il core e per l'RDP, e il mock HTML/CSS andrebbe ricostruito.
- **Mock**: quello in `docs/ui/project/` è definitivo; le correzioni elencate in `docs/ui/NOTE.md` non vengono
  riportate nel mock ma integrate direttamente nella specifica della UI (`docs/SPEC-UI.md`).
- **RDP integrato**: resta un passo successivo alla UI (DEC-013), ma la UI prevede fin da ora lo spazio per il
  pannello RDP nelle tab di sessione, così l'aggiunta non cambierà la struttura.
- **Metodo**: come per la FASE 1/2a: specifica (`docs/SPEC-UI.md`) + prompt PM, implementazione con sottoagenti in
  worktree, revisione indipendente prima di ogni merge, quality gate estesi al frontend.

### DEC-028 – Esito dei punti incerti della ricognizione (prove di UI-WP0, decise dal PM)
1. **Sidecar e `--config`**: confermato SPEC-UI §4.10. `externalBin` solo in `tauri.bundle.json`; nel bundle
   `Contents/MacOS/portolano` sta accanto all'app, entrambi firmati ad hoc con hardened runtime, e
   `codesign --verify --deep --strict` passa. I gate Rust passano senza la cartella `binaries/`.
2. **Askpass su `/dev/tty` nella PTY**: funziona (conferma della host key e prompt sconosciuto senza eco, test
   `crates/portolano-app/tests/askpass_tty.rs`). Il client askpass però stampa il prompt nascosto **prima** di
   spegnere l'eco (comportamento di `rpassword`), quindi un input incollato subito dopo il prompt può comparire in
   chiaro. **Decisione**: in UI-WP3 il client askpass spegne l'eco prima di scrivere il prompt e la riaccende anche in
   caso di errore, con un test; il comportamento resta identico per la CLI.
3. **`'unsafe-inline'` negli stili**: resta necessario solo in `style-src` (xterm 6 con renderer DOM produce 14
   violazioni senza; i colori del tema si perdono). `script-src` resta `'self'`. Si aggiunge
   `"dangerousDisableAssetCspModification": ["style-src"]`, perché il nonce che Tauri aggiungerebbe a `style-src`
   farebbe ignorare `'unsafe-inline'` a WebKit. Da verificare nel bundle reale nella verifica finale.
4. **TIFF e PDF in `<img>`** da `portolano-icon://`: WKWebView li mostra (provato in `tauri dev`). Il segnaposto serve
   solo per formati non riconosciuti o file illeggibili.
5. **Permessi**: `core:event:default`, `core:window:default`, `core:window:allow-start-dragging`,
   `opener:allow-open-url` limitato a `http(s)://*` e i permessi dei comandi dell'app. **Nessun permesso del plugin
   dialog**: tutti i dialoghi dell'interfaccia sono componenti React e i percorsi li sceglie Rust (§4.6), quindi
   `dialog:allow-message` si toglie. Modalità d'errore di tauri-specta: `ErrorHandlingMode::Result`.
6. **Prestazioni del vetro**: misura preliminare, non affidabile (la webview ha smesso di produrre frame dopo 51
   frame): canvas animato e 3 pannelli `blur(18px)` a 1280×800, DPR 2, p50 33 ms, p95 41 ms. **Decisione**: UI-WP1
   ferma l'animazione dello sfondo quando la finestra non è visibile o non ha il focus, rispetta
   `prefers-reduced-motion` e l'interruttore di §6, e limita l'animazione a 30 fps; il PM rimisura con Web Inspector
   al primo checkpoint visivo prima di confermare i valori di blur.
7. **Bundle**: `Portolano.app` 18,5 MiB, `.dmg` 6,0 MiB. La prova del Keychain con l'app firmata ad hoc resta un test
   manuale dell'utente (SPEC-UI §9).
- **Contatori `usize`** nei tipi di ops e import: specta-typescript li rifiuta. Si annotano con
  `#[cfg_attr(feature = "specta", specta(type = u32))]` (nessun DTO duplicato; i conteggi reali sono molto sotto il
  limite) e un test dell'app esporta tutti i tipi di §3.5, così un tipo non esportabile emerge subito. Non si usa
  `dangerously_cast_bigints_to_number()`.
- **Icone personalizzate**: lo schema `portolano-icon://` serve solo file regolari (niente symlink), entro un limite
  di dimensione, e solo se lo SHA-256 dei byte coincide con il nome; controllo dell'hash e tipi MIME sono quelli del
  core, non duplicati nell'app (revisione di UI-WP0).
- **CSP in sviluppo**: in `tauri dev` non è stato possibile verificare che `devCsp` venga applicata; la CSP valida è
  quella del bundle, verificata nella verifica finale. Gli e2e servono già le pagine con la CSP di `tauri.conf.json` e
  falliscono a ogni violazione.

### DEC-006 – Conflitti fra documenti già risolti dalla SPEC (nessuna domanda)
- `CredentialMode` assente → `Inherit` (SPEC §8.3), non `none` come in `royal-format.md` §5.1.
- Service Keychain `dev.portolano` (SPEC §5.2), non `portolano`.
