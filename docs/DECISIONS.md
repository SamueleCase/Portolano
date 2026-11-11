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

### DEC-006 – Conflitti fra documenti già risolti dalla SPEC (nessuna domanda)
- `CredentialMode` assente → `Inherit` (SPEC §8.3), non `none` come in `royal-format.md` §5.1.
- Service Keychain `dev.portolano` (SPEC §5.2), non `portolano`.
