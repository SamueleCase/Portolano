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
- **Motivo**: senza questa forma la passphrase delle chiavi nel vault non verrebbe mai servita dall'askpass.

### DEC-006 – Conflitti fra documenti già risolti dalla SPEC (nessuna domanda)
- `CredentialMode` assente → `Inherit` (SPEC §8.3), non `none` come in `royal-format.md` §5.1.
- Service Keychain `dev.portolano` (SPEC §5.2), non `portolano`.
