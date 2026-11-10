# Portolano – Specifica di implementazione (FASE 1 + FASE 2a)

> **Documento-contratto.** È la fonte di verità per chi implementa: il PM e i sottoagenti.
> Si legge insieme a [`royal-format.md`](royal-format.md) (formato Royal) e a [`fase0-stack.md`](fase0-stack.md)
> (motivazioni dello stack). Se questa specifica contraddice gli altri due documenti, vale questa.
> Le deviazioni si annotano in `docs/DECISIONS.md` con data e motivo.
>
> Lingua: documenti e messaggi utente in **italiano**; codice, identificatori e commenti in **inglese**.

---

## 0. Obiettivo e decisioni già prese

Portolano è un gestore di connessioni remote (SSH, RDP) ispirato a Royal TSX. Questa specifica copre:

- **FASE 1**: modello dati, persistenza versionata, vault credenziali, CLI di gestione dell'albero.
- **FASE 2a**: `portolano connect` per SSH e RDP, importer Royal TS/TSX con report.

La **UI (FASE 2) è fuori scope**, ma il core va esposto come libreria pulita perché Tauri la userà.

Decisioni approvate dall'utente, **non da ridiscutere**:

| # | Decisione |
|---|---|
| D1 | Stack **Rust** (workspace Cargo). UI futura **Tauri v2**. |
| D2 | **Formato proprio** (JSON versionato). Royal è solo il primo *importer*; altri seguiranno con lo stesso trait. |
| D3 | Le password cifrate Royal **non si decifrano**: algoritmo proprietario e non documentato. L'importer importa la struttura e **chiede le password in modo interattivo** (opzionale), poi le salva nel vault. Il ciphertext Royal **non** si conserva da nessuna parte. |
| D4 | RDP: client predefinito **FreeRDP 3**, con password passata **via stdin**. L'utente vuole che una credenziale salvata faccia login **senza richiederla**. *Windows App* resta un fallback, che però chiede la password. |
| D5 | SSH: si usa l'**OpenSSH di sistema**. Le password arrivano a ssh tramite `SSH_ASKPASS` servito da Portolano, mai via argv, env o file. |
| D6 | Segreti solo nel **vault**: Keychain di macOS di default, oppure vault su file cifrato. Il JSON contiene solo riferimenti opachi. |

---

## 1. Regole non negoziabili

1. **Nessun segreto** (password, passphrase, chiavi private, master password) può comparire in:
   - file del documento, report, log, output della CLI, messaggi d'errore, `Debug` dei tipi;
   - argv dei processi figli, variabili d'ambiente dei figli, file temporanei;
   - test, fixture, commit.
2. I segreti in memoria usano il tipo `Secret` (§5.1): azzerato al drop, `Debug` = `Secret(***)`, nessun `Display`, nessun `Serialize`.
3. **Dati di test**: solo host di esempio (`*.example.com`, `192.0.2.0/24`, `198.51.100.0/24`, RFC 5737) e utenti finti.
   La master password di test del vault su file è la costante `"test-only-master-password"`, ammessa solo nei test.
4. `samples/` (documenti Royal reali) è in `.gitignore`. I test che li usano:
   - sono `#[ignore]` e si attivano con `PORTOLANO_SAMPLES_DIR`;
   - stampano **solo conteggi**, mai nomi, host o utenti.
5. Architettura a moduli: `core` non conosce SSH, RDP o Royal. Un nuovo protocollo o importer si aggiunge
   **senza modificare `core`**.
6. Ogni crate compila con `cargo clippy --all-targets -- -D warnings` e `cargo fmt --check` pulito.
7. Il codice rispetta gli **standard del §11.3** ed è sottoposto a revisione prima di ogni merge.
   Ogni dubbio sulla SPEC si risolve con una domanda, non con un'ipotesi.

---

## 2. Architettura

```
Portolano/
├── Cargo.toml                    # [workspace] + [workspace.dependencies] (tutte le dipendenze, §2.2)
├── crates/
│   ├── portolano-core/           # modello, albero, risoluzione credenziali, store JSON, migrazioni,
│   │                             # icone, vault, config. Nessuna dipendenza da connectors/import/cli.
│   ├── portolano-connectors/     # trait Protocol + registry; ssh; rdp; askpass; esecuzione; terminali
│   ├── portolano-import/         # trait Importer + report + registry; modulo royal
│   └── portolano-cli/            # binario `portolano` (clap). Sottile: solo parsing, I/O e output.
├── docs/  (SPEC.md, royal-format.md, fase0-stack.md, DECISIONS.md, progress.md)
├── scripts/smoke.sh              # test end-to-end manuale/CI (§11.4)
└── README.md
```

Grafo delle dipendenze: `cli → {core, connectors, import}`, `connectors → core`, `import → core`.

### 2.1 Struttura dei file (assegnazione ai pacchetti di lavoro, §12)

```
crates/portolano-core/src/
  lib.rs            WP0   re-export pubblici
  error.rs          WP0   Error, VaultError, Issue
  model.rs          WP0   tutti i tipi del §3 (solo definizioni + costruttori + serde)
  secret.rs         WP0   Secret, SecretRef
  tree.rs           WP1   operazioni sull'albero (§4.1)
  path.rs           WP1   sintassi e risoluzione dei percorsi (§4.2)
  resolve.rs        WP1   risoluzione credenziali (§4.3)
  validate.rs       WP1   validazione strutturale (§4.4)
  store.rs          WP2   load/save atomico, lock (§6)
  migrate.rs        WP2   migrazioni schema (§6.3)
  icons.rs          WP2   IconStore (§6.4)
  config.rs         WP2   AppConfig (§9)
  vault/mod.rs      WP3   trait Vault + open_vault() (§5)
  vault/memory.rs   WP3
  vault/keychain.rs WP3
  vault/file.rs     WP3
crates/portolano-connectors/src/   WP4 (tutto)
  lib.rs, protocol.rs, registry.rs, plan.rs, env.rs, exec.rs, terminal.rs,
  ssh/{mod.rs, settings.rs, askpass.rs}, rdp/{mod.rs, settings.rs, freerdp.rs, windows_app.rs}
crates/portolano-import/src/        WP5 (tutto)
  lib.rs, report.rs, prompter.rs, registry.rs,
  royal/{mod.rs, sniff.rs, xml.rs, map.rs, defaults.rs}
crates/portolano-cli/src/           WP6 (tutto)
  main.rs, args.rs, output.rs, commands/{init,tree,list,show,add,edit,mv,delete,secret,connect,import,validate,config,askpass}.rs
```

WP0 crea **tutti** i file con le firme pubbliche di questa specifica e corpi `todo!()`, così il workspace
compila subito. I WP successivi riempiono **solo i propri file**.

### 2.2 Dipendenze (in `[workspace.dependencies]`, dichiarate tutte da WP0)

Si usa l'ultima major stabile al momento dell'implementazione; nel dubbio va verificata con `cargo add --dry-run`.

`serde` (derive), `serde_json`, `uuid` (v4, serde), `chrono` (serde), `thiserror`, `anyhow` (solo cli),
`clap` (derive, env, wrap_help), `keyring` (feature native per macOS, Windows e Linux; va verificato il nome
delle feature nella versione scelta), `argon2`, `chacha20poly1305`, `rand` / `getrandom`, `base64`, `zeroize`,
`directories`, `toml`, `quick-xml`, `sha2`, `hex`, `tempfile`, `fd-lock`, `rpassword`, `which`, `regex`,
`shell-words` (quoting per `--print`), `tracing`, `tracing-subscriber`.
Dev: `assert_cmd`, `predicates`, `pretty_assertions`.

Un sottoagente che ha bisogno di una dipendenza nuova la aggiunge **solo** al `Cargo.toml` del proprio crate
e la segnala al PM nel report finale.

---

## 3. Modello dati (`portolano-core::model`) – contratto esatto

Albero **annidato** sia in memoria sia su disco. L'ordine dei figli è l'ordine nell'array (manuale, non alfabetico).

```rust
pub const FORMAT_ID: &str = "portolano";
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub format: String,                 // sempre "portolano"
    pub schema_version: u32,
    pub id: Uuid,
    pub name: String,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    #[serde(default)]
    pub nodes: Vec<Node>,               // figli della radice
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: Uuid,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub notes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub icon: Option<Icon>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub color: Option<String>, // "#RRGGBB"
    #[serde(default, skip_serializing_if = "is_false")]        pub favorite: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]   pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]   pub custom_properties: Vec<CustomProperty>,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,     // campi importati non mappati, chiavi "royal.<Campo>"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceInfo>,
    #[serde(flatten)]
    pub kind: NodeKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NodeKind {
    Folder(Folder),
    Connection(Connection),
    Credential(Credential),
    Unsupported(Unsupported),
}

pub struct Folder {
    #[serde(default)] pub children: Vec<Node>,
    #[serde(default, skip_serializing_if = "CredentialSpec::is_inherit")]
    pub credential: CredentialSpec,               // default per i discendenti; default Inherit
}

pub struct Connection {
    pub protocol: String,                         // "ssh", "rdp", … (id del Protocol)
    pub host: String,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub port: Option<u16>, // None = default del protocollo
    #[serde(default, skip_serializing_if = "CredentialSpec::is_inherit")]
    pub credential: CredentialSpec,
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub settings: serde_json::Map<String, serde_json::Value>, // proprietà del Protocol (§7); il core non le interpreta
}

pub struct Credential {
    #[serde(default, skip_serializing_if = "Option::is_none")] pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub domain: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub password: Option<SecretRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub private_key: Option<PrivateKey>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub passphrase: Option<SecretRef>,
}

#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PrivateKey {
    File { path: String },                        // percorso su disco (può iniziare con ~)
    Vault { secret: SecretRef },                  // contenuto della chiave nel vault
}

#[derive(Default)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum CredentialSpec {
    #[default] Inherit,
    None,
    Prompt,
    Inline {
        #[serde(default, skip_serializing_if = "Option::is_none")] username: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")] domain: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")] password: Option<SecretRef>,
    },
    ById { #[serde(rename = "$ref")] id: Uuid },
    ByName { name: String },
}

#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Icon {
    Builtin { name: String },                     // set Portolano (FASE 2)
    Royal { name: String },                       // percorso nella IconLibrary di Royal, es. "Flat/Hardware/Server"
    Custom { sha256: String, mime: String },      // file in IconStore (§6.4)
}

pub struct CustomProperty {
    pub name: String,
    #[serde(flatten)] pub value: PropertyValue,
}
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PropertyValue { Text { value: String }, Secret { secret: SecretRef } }

pub struct Unsupported {
    pub source_type: String,                      // es. "RoyalVNCConnection"
    pub raw: String,                              // frammento originale (XML) completo
}

pub struct SourceInfo {
    pub system: String,                           // "royal"
    pub source_type: String,                      // "RoyalSSHConnection"
    #[serde(default, skip_serializing_if = "Option::is_none")] pub source_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub file: Option<String>, // solo nome file, non il percorso
    pub imported_at: DateTime<Utc>,
}
```

**Convenzione dei riferimenti `"$ref"`**: *ogni* riferimento da un nodo a un altro nodo, ovunque si trovi
(anche dentro `settings` dei protocolli), si serializza come oggetto con chiave `"$ref": "<uuid>"`.
Così il core trova i riferimenti in modo generico, senza conoscere i protocolli (§4.4).

Costruttori richiesti: `Document::new(name)`, `Node::folder(name)`, `Node::connection(name, protocol, host)`,
`Node::credential(name)`, `Node::touch(&mut self)` (aggiorna `modified`).
Helper: `Node::is_folder()`, `as_folder()`, `as_connection()`, `as_credential()` e le versioni `_mut`.

### 3.1 Esempio di file (`portolano.json`)

```json
{
  "format": "portolano",
  "schema_version": 1,
  "id": "6f1c…",
  "name": "Portolano",
  "created": "2026-11-10T10:00:00Z",
  "modified": "2026-11-10T10:05:00Z",
  "nodes": [
    {
      "id": "a1…", "name": "Clienti", "created": "…", "modified": "…",
      "type": "folder",
      "credential": { "mode": "by_id", "$ref": "c3…" },
      "children": [
        {
          "id": "b2…", "name": "web01", "created": "…", "modified": "…",
          "icon": { "kind": "royal", "name": "Flat/Hardware/Server" },
          "type": "connection", "protocol": "ssh", "host": "web01.example.com", "port": 2222,
          "settings": { "jump_hosts": [ { "$ref": "d4…" } ], "agent_forwarding": true }
        },
        {
          "id": "c3…", "name": "Admin ACME", "created": "…", "modified": "…",
          "type": "credential", "username": "admin", "domain": "ACME",
          "password": "8e0d6a2c-…"
        }
      ]
    }
  ]
}
```

---

## 4. Logica del core

### 4.1 Albero (`tree.rs`), metodi su `Document`

```rust
pub fn find(&self, id: Uuid) -> Option<&Node>;
pub fn find_mut(&mut self, id: Uuid) -> Option<&mut Node>;
pub fn parent_of(&self, id: Uuid) -> Option<Option<Uuid>>;      // None = non trovato, Some(None) = radice
pub fn ancestors(&self, id: Uuid) -> Vec<Uuid>;                 // dal padre verso la radice
pub fn children(&self, parent: Option<Uuid>) -> Result<&[Node]>;
pub fn insert(&mut self, parent: Option<Uuid>, position: Option<usize>, node: Node) -> Result<()>;
pub fn remove(&mut self, id: Uuid) -> Result<Node>;             // stacca il sottoalbero
pub fn move_node(&mut self, id: Uuid, new_parent: Option<Uuid>, position: Option<usize>) -> Result<()>;
pub fn walk(&self) -> impl Iterator<Item = (usize, &Node)>;     // pre-order, (profondità, nodo)
pub fn secret_refs_in(&self, id: Uuid) -> Vec<SecretRef>;       // tutti i SecretRef nel sottoalbero
```

Regole:
- `insert`: il genitore deve essere un `Folder`. Tutti gli `id` del sottoalbero inserito devono essere
  **univoci nel documento**. `position` `None` = in coda; un valore oltre la fine viene ridotto all'ultima posizione.
- `move_node`: vietato spostare un nodo in sé stesso o in un suo discendente (`Error::InvalidMove`).
  `position` è l'indice nella lista **finale**, quindi il riordino nello stesso padre funziona in modo intuitivo.
  Aggiorna `modified` del nodo spostato.
- Profondità illimitata. Il codice può essere ricorsivo: si assumono profondità < 1000.

### 4.2 Percorsi (`path.rs`)

- Sintassi: segmenti separati da `/`; `/` iniziale facoltativo; `/` da solo = radice.
  Nei nomi `\/` indica uno slash letterale e `\\` un backslash.
- `@<uuid>` o `@<prefisso esadecimale ≥ 6>` indirizza un nodo per id, in qualsiasi punto.
- `pub fn path_of(&self, id) -> Option<String>`: restituisce il percorso con l'escape applicato.
- `pub fn resolve_path(&self, path: &str) -> Result<Uuid>`:
  - confronto esatto e case-sensitive;
  - se non trova nulla, ripete il confronto senza distinguere maiuscole e minuscole;
  - se trova più nodi → `Error::Ambiguous { query, candidates: Vec<(Uuid, String /*path*/)> }`.
- `pub fn resolve_parent_path(&self, path) -> Result<(Option<Uuid>, String /*ultimo segmento*/)>`
  per i comandi `add`. Con `create_missing: bool` crea le cartelle intermedie (opzione `-p`).

### 4.3 Risoluzione credenziali (`resolve.rs`)

```rust
pub struct ResolvedCredential {
    pub origin: CredentialOrigin,          // None | Prompt | Inline { node: Uuid } | Credential { node: Uuid }
    pub username: Option<String>,
    pub domain: Option<String>,
    pub password: Option<SecretRef>,
    pub private_key: Option<PrivateKey>,
    pub passphrase: Option<SecretRef>,
}
pub fn resolve_credential(&self, node_id: Uuid) -> Result<ResolvedCredential>;   // usa la spec del nodo
pub fn resolve_spec(&self, context: Uuid, spec: &CredentialSpec) -> Result<ResolvedCredential>;
```

Algoritmo, con `cur = spec` e `ctx = context`:
1. `Inherit` → passa al padre di `ctx`. Se il padre è la radice, il risultato è `origin = None`.
   Altrimenti `cur = folder.credential`, `ctx = padre`, e si ripete.
2. `None` → `origin None`. `Prompt` → `origin Prompt`.
3. `Inline` → si copiano i campi.
4. `ById` → il nodo deve esistere ed essere `Credential`. Altrimenti `Error::DanglingReference`, o
   `Error::NotACredential` se esiste ma è di altro tipo.
5. `ByName` → si cercano i `Credential` con nome identico in tutto il documento.
   - 0 risultati → `Error::NotFound`;
   - più di 1 → `Error::Ambiguous`;
   - prima si confronta in modo case-sensitive, poi senza distinguere maiuscole e minuscole.

### 4.4 Riferimenti e validazione (`validate.rs`)

- `pub fn references_to(&self, target: Uuid) -> Vec<Uuid>`: i nodi che contengono `{"$ref": target}`
  in qualsiasi punto. Si trovano serializzando il nodo, *senza figli*, in `Value` e scandendolo.
  Contano anche i nodi con `ByName` uguale al nome del target, se il target è una credenziale.
- `pub fn validate(&self) -> Vec<Issue>`, con
  `Issue { severity: Error|Warning, node: Option<Uuid>, code: &'static str, message: String }`:
  - `duplicate_id`;
  - `dangling_ref`: un `$ref` che punta a un nodo inesistente;
  - `credential_ref_not_credential`;
  - `by_name_unresolved` e `by_name_ambiguous`;
  - `empty_host`;
  - `unknown_protocol`: è un Warning e viene verificato dal registry della CLI, non dal core.
- La CLI aggiunge i controlli specifici dei protocolli (`Protocol::validate`) e quelli sul vault
  (`missing_secret`: un `SecretRef` senza voce nel vault).

---

## 5. Segreti e vault

### 5.1 Tipi (`secret.rs`, WP0)

```rust
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)] #[serde(transparent)]
pub struct SecretRef(String);              // uuid v4 in forma testuale; new() ne genera uno
impl SecretRef { pub fn new() -> Self; pub fn as_str(&self) -> &str; }

pub struct Secret(zeroize::Zeroizing<String>);
impl Secret { pub fn new(s: String) -> Self; pub fn expose(&self) -> &str; }
impl fmt::Debug for Secret { /* "Secret(***)" */ }
// NON implementare Display, Serialize, Deserialize, ToString.
```

### 5.2 Trait e backend (`vault/`, WP3)

```rust
pub trait Vault: Send + Sync {
    fn backend(&self) -> &'static str;                                  // "keychain" | "file" | "memory"
    fn get(&self, r: &SecretRef) -> Result<Option<Secret>, VaultError>;
    fn set(&self, r: &SecretRef, s: &Secret) -> Result<(), VaultError>;
    fn delete(&self, r: &SecretRef) -> Result<(), VaultError>;          // idempotente
    fn contains(&self, r: &SecretRef) -> Result<bool, VaultError> { Ok(self.get(r)?.is_some()) }
}
pub enum VaultSelection { Keychain, File { path: PathBuf }, Memory }
pub trait MasterPasswordSource { fn master_password(&self, creating: bool) -> Result<Secret, VaultError>; }
pub fn open_vault(sel: &VaultSelection, pw: &dyn MasterPasswordSource) -> Result<Box<dyn Vault>, VaultError>;
```

- **Keychain** (`keyring`): service `dev.portolano`, account = `SecretRef`. È il default su macOS.
  Nota per il README: in sviluppo i binari non firmati possono far comparire il prompt
  "consenti l'accesso al portachiavi" dopo ogni ricompilazione, ed è normale.
  Su Windows e Linux si usano i backend nativi di `keyring`.
- **File**: `<data_dir>/vault.json`, riscritto in modo atomico ad ogni `set` o `delete`. Formato:
  ```json
  { "format": "portolano-vault", "version": 1,
    "kdf": { "alg": "argon2id", "m_cost_kib": 65536, "t_cost": 3, "p_cost": 1, "salt": "<b64>" },
    "cipher": "xchacha20poly1305", "nonce": "<b64 24B>", "ciphertext": "<b64>" }
  ```
  - Il plaintext è un JSON `{ "<secretref>": "<b64 del segreto>" }`.
  - Ad ogni scrittura si generano un nonce nuovo e casuale e un salt fisso per il file.
  - Con una master password errata si ottiene `VaultError::BadMasterPassword`.
  - La master password viene da prompt nascosto, oppure dalla variabile `PORTOLANO_VAULT_PASSWORD`
    (documentata come "solo per automazione e test").
  - Nei test sono ammessi parametri KDF ridotti tramite un costruttore `#[cfg(test)]` o un costruttore pubblico
    `with_kdf_params`.
- **Memory**: HashMap dietro Mutex, per i test e per `--vault memory` (opzione nascosta della CLI).

---

## 6. Persistenza (`store.rs`, `migrate.rs`, `icons.rs`, WP2)

### 6.1 Posizioni

`directories::ProjectDirs::from("dev", "portolano", "Portolano")`. Su macOS diventa
`~/Library/Application Support/dev.portolano.Portolano/`:
- documento di default `portolano.json`
- `config.toml`
- `vault.json`, solo per il backend file.

Il documento si può scegliere con `--file` o con `PORTOLANO_FILE`. Il caso tipico è un file in un repo git dell'utente.

### 6.2 Store

```rust
pub struct Store { path: PathBuf, /* lock */ }
impl Store {
    pub fn open(path: impl Into<PathBuf>) -> Result<Store>;   // acquisisce un lock esclusivo su "<file>.lock" (fd-lock)
    pub fn create(path, name: &str) -> Result<(Store, Document)>; // errore se il file esiste
    pub fn load(&self) -> Result<Document>;
    pub fn save(&self, doc: &mut Document) -> Result<()>;     // aggiorna doc.modified
    pub fn icons(&self) -> IconStore;
}
```

- `load`:
  1. legge il JSON come `Value` e verifica `format == "portolano"`;
  2. se `schema_version` è maggiore di quello supportato → `Error::UnsupportedSchema { found, supported }`;
  3. se è minore, applica le migrazioni;
  4. deserializza nei tipi;
  5. se `validate()` trova `duplicate_id` → errore.
- `save`:
  - JSON pretty con indentazione di 2 spazi e newline finale;
  - scrive `<file>.tmp-<rand>` nella stessa directory, esegue `fsync` e poi `rename`;
  - prima di sovrascrivere copia il file precedente in `<file>.bak`;
  - l'output è **deterministico**: stesso documento, stessi byte.

### 6.3 Migrazioni

```rust
pub type Migration = fn(serde_json::Value) -> Result<serde_json::Value>;
pub const MIGRATIONS: &[(u32 /*from*/, Migration)] = &[];   // v1 è la prima versione
pub fn migrate(v: Value, from: u32, to: u32, table: &[(u32, Migration)]) -> Result<Value>;
```

Ogni migrazione porta da `n` a `n+1` e aggiorna `schema_version`. I test usano una tabella sintetica
(v1→v2→v3) per verificare l'ordine, le versioni mancanti e i casi di errore.

### 6.4 Icone

`IconStore` salva le immagini custom in `<dir del doc>/<stem del doc>.icons/<sha256>.<ext>`.

```rust
pub fn add(&self, bytes: &[u8], mime: &str) -> Result<Icon>;   // deduplica per hash
pub fn path_for(&self, icon: &Icon) -> Option<PathBuf>;
pub fn gc(&self, doc: &Document) -> Result<usize>;            // rimuove le immagini non più usate
```

MIME ammessi: png, jpeg, gif, tiff, svg+xml, pdf. Il tipo si riconosce dai magic byte e l'estensione si deriva dal MIME.

---

## 7. Connettori (`portolano-connectors`, WP4)

### 7.1 Trait e registry

```rust
pub trait Protocol: Send + Sync {
    fn id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn default_port(&self) -> u16;
    fn validate_settings(&self, settings: &serde_json::Map<String, Value>) -> Result<(), String>;
    fn validate(&self, doc: &Document, node: &Node) -> Vec<Issue>;
    fn summary(&self, doc: &Document, node: &Node) -> String;  // es. "admin@web01.example.com:22" (per tree/list)
    fn plan(&self, ctx: &PlanContext) -> Result<LaunchPlan, ConnectError>;  // PURO: nessun I/O, nessun segreto
}
pub struct Registry;   // Registry::with_defaults() registra ssh, rdp; get(id); ids()
pub struct PlanContext<'a> {
    pub doc: &'a Document, pub node_id: Uuid, pub config: &'a AppConfig,
    pub env: &'a Environment, pub options: &'a ConnectOptions,
}
pub struct ConnectOptions { pub new_window: bool, pub rdp_client: Option<RdpClient>, pub wait: bool }
pub struct Environment {   // risultato di Environment::discover(&config); nei test si costruisce a mano
    pub os: Os, pub ssh: Option<PathBuf>, pub ssh_add: Option<PathBuf>,
    pub freerdp: Option<PathBuf>, pub windows_app: Option<PathBuf>, pub self_exe: PathBuf,
}
pub struct LaunchPlan { pub steps: Vec<LaunchStep>, pub warnings: Vec<String> }
pub enum LaunchStep {
    SshAddKey { key: SecretRef, passphrase: Option<SecretRef>, lifetime_s: u32 },
    WriteTempFile { name: String, contents: String /* mai segreti */, mode: u32 },
    Run { program: PathBuf, args: Vec<String> /* "{temp:<name>}" sostituito */,
          env: Vec<(String, String)> /* mai segreti */,
          stdin_secret: Option<SecretRef>, askpass: Option<AskpassSpec>, wait: bool },
    OpenInTerminal { argv: Vec<String> },
}
pub fn render(plan: &LaunchPlan) -> String;   // per --print: argv quotato, i segreti come <vault:ref>
pub fn execute(plan: &LaunchPlan, vault: &dyn Vault, config: &AppConfig) -> Result<i32, ConnectError>;
```

Le impostazioni tipizzate di ogni protocollo (`SshSettings`, `RdpSettings`) hanno `serde(default)`
e `deny_unknown_fields`. `validate_settings` le deserializza dalla mappa `Connection.settings`.

### 7.2 SSH

```rust
pub struct SshSettings {
    pub jump_hosts: Vec<JumpHost>,          // in ordine: primo salto → ultimo
    pub agent_forwarding: bool, pub x11: bool, pub x11_trusted: bool, pub compression: bool,
    pub keepalive_s: Option<u32>,
    pub options: BTreeMap<String, String>,  // passate come -o Key=Value
}
#[serde(untagged)]
pub enum JumpHost { Ref { #[serde(rename = "$ref")] id: Uuid }, Address { address: String /* [user@]host[:port] */ } }
```

Il piano (in modalità `Here`):
- `ssh`, poi `-p <port>` e, se c'è uno username, `-l <user>`.
- **Chiave**:
  - `PrivateKey::File{path}` → `-i <path espanso> -o IdentitiesOnly=yes`.
  - `PrivateKey::Vault` → si aggiunge prima uno step `SshAddKey` (`ssh-add -t 300 -`, con la chiave su stdin e la
    passphrase via askpass). Se l'agent non è disponibile si restituisce un errore chiaro.
    **Mai** un file temporaneo con la chiave.
- **Jump**: `-J j1,j2,…`.
  - Ogni `Ref` deve puntare a una connessione `ssh` e diventa `user@host:port` usando la sua credenziale risolta.
  - Le catene si appiattiscono: i jump del jump vengono prima.
  - I cicli producono `ConnectError::JumpCycle`.
- `-A`, `-X` o `-Y`, `-C`, `-o ServerAliveInterval=N`, poi le `-o K=V`, e per ultimo `host`.
- **Password**: se ci sono segreti per l'host di destinazione o per i jump, si aggiungono
  `-o NumberOfPasswordPrompts=1` e un `AskpassSpec`:
  ```rust
  pub struct AskpassSpec { pub entries: Vec<AskpassEntry> }
  pub struct AskpassEntry { pub matcher: PromptMatcher, pub secret: SecretRef }
  pub enum PromptMatcher { Password { user: String, host: String }, Passphrase { key_path: String } }
  ```

**Protocollo askpass** (`ssh/askpass.rs`, lato server ed `exec.rs`):
1. `execute` crea una directory temporanea `0700` con un socket Unix `askpass.sock` e un token casuale da
   32 byte (hex). Avvia un thread server che conserva in memoria, come `Secret`, i segreti necessari letti dal vault.
2. Variabili d'ambiente del figlio `ssh`: `SSH_ASKPASS=<self_exe>`, `SSH_ASKPASS_REQUIRE=force`,
   `PORTOLANO_ASKPASS=1`, `PORTOLANO_ASKPASS_SOCK=<path>`, `PORTOLANO_ASKPASS_TOKEN=<token>`.
   Il token non è un segreto dell'utente: è monouso e vale per una sola sessione.
3. Lato client, `portolano` in `main` controlla **prima di clap** se `PORTOLANO_ASKPASS=1`. In quel caso:
   - si collega al socket e invia `{"token":…, "prompt": argv[1]}\n`;
   - riceve `{"answer": "…"}` e lo stampa su stdout seguito da newline, con exit 0;
   - oppure riceve `{"fallback": true}`.
4. Il server confronta il prompt:
   - `^(?P<user>[^@\s]+)@(?P<host>[^']+)'s password: ?$`
   - `^\((?P<user>[^@\s]+)@(?P<host>[^)]+)\) Password: ?$` (keyboard-interactive)
   - `^Enter passphrase for key '(?P<path>.+)': ?$`

   Ogni voce viene servita **una sola volta**: alla seconda richiesta si risponde con fallback, così una password
   sbagliata non va in loop.
5. **Fallback**: il client apre `/dev/tty` e chiede direttamente all'utente.
   - Se il prompt contiene `(yes/no` si usa input con eco: è la conferma della host key.
   - Altrimenti l'input è nascosto.
   - Se non c'è un TTY, exit 1.
6. Alla fine di `ssh` si chiudono il socket e la directory e i segreti vengono azzerati.
   Il codice d'uscita di `portolano connect` è quello di `ssh`.

**Nuova finestra** (`--new-window`, `terminal.rs`): uno step `OpenInTerminal` esegue
`<self_exe> connect --file <doc> @<id>`. Il comando non contiene segreti e il processo figlio riapre il vault.

| `terminal.app` | Comando |
|---|---|
| `terminal` (default) | `osascript -e 'tell application "Terminal" to do script "<cmd quotato>"' -e 'tell application "Terminal" to activate'` |
| `iterm` | AppleScript `create window with default profile command "<cmd>"` |
| `ghostty` | `open -na Ghostty --args -e <argv>` |
| `custom` | `terminal.command` con il segnaposto `{cmd}` |
| Linux | `x-terminal-emulator -e <argv>` |

### 7.3 RDP

```rust
pub struct RdpSettings {
    pub resolution: Resolution,             // tag "mode": fit (default) | fullscreen | fixed { width, height }
    pub scale: Option<u32>,                 // 100 | 140 | 180
    pub color_depth: Option<u8>,            // 15 | 16 | 24 | 32
    pub gateway: Option<RdpGateway>,        // { host, port: Option<u16>, credential: CredentialSpec }
    pub multimon: bool, pub admin: bool,
    pub clipboard: bool,                    // default true
    pub wol_mac: Option<String>,            // solo memorizzato in questa fase
}
pub enum RdpClient { FreeRdp, WindowsApp }
```

**Scelta del client**: `ConnectOptions.rdp_client` ha la precedenza su `config.rdp.client` (`auto|freerdp|windows-app`).
In `auto` si usa FreeRDP se `env.freerdp` è presente, altrimenti Windows App con warning
"la password verrà richiesta"; se non c'è nessuno dei due, errore con suggerimento `brew install freerdp`.

**FreeRDP**: si cercano in ordine `sdl-freerdp3`, `sdl-freerdp`, `xfreerdp3`, `xfreerdp` nel PATH e in
`/opt/homebrew/bin` e `/usr/local/bin`. Argomenti:
- `/v:<host>:<port>`, `/u:<user>`, `/d:<domain>`;
- `/cert:<config.rdp.cert, default tofu>`;
- risoluzione: `/dynamic-resolution` (fit), `/f` (fullscreen) oppure `/size:WxH`;
- `/scale:N`, `/bpp:N`, `/multimon`, `/admin`, `+clipboard` / `-clipboard`.

Password:
- se c'è un segreto, si passa **via stdin** (`stdin_secret`) con l'opzione di FreeRDP per leggere le credenziali
  da stdin (`/from-stdin[:force]`). **Il comportamento esatto va verificato** con `/help` e un test reale
  sulla versione installata;
- `execute` scrive la password con un newline e chiude stdin;
- **mai** `/p:` in argv.

Gateway:
- si costruisce `/gw:g:<host>[:<port>],u:<user>,d:<domain>`;
- se la credenziale del gateway coincide con quella di destinazione si usa l'opzione FreeRDP per riusare le
  credenziali, se esiste. Altrimenti la password del gateway la chiede FreeRDP in modo interattivo, con un warning
  nel piano;
- **mai** `p:` in argv.

Di default `wait=false`: il processo si stacca dopo aver scritto stdin. Con `--wait` si attende la chiusura.

**Windows App**: va cercata in `/Applications/Windows App.app`, con fallback `Microsoft Remote Desktop.app`.
- Si genera `<name>.rdp` in una directory temporanea per utente, con permessi `0600`.
- Contenuto: `full address:s:host:port`, `username:s:DOMAIN\user`, `gatewayhostname:s:…`,
  `gatewayusagemethod:i:1`, `screen mode id:i:1|2`, `desktopwidth:i:`, `desktopheight:i:`,
  `session bpp:i:`, `redirectclipboard:i:`, `use multimon:i:`, `administrative session:i:`.
  **Nessuna password.**
- Lo si apre con `open -a <app> <file>`.
- A ogni `connect` si cancellano i `.rdp` più vecchi di 10 minuti.

### 7.4 Test richiesti (WP4)

- Test sui piani: SSH con password, con chiave su file, con chiave nel vault, jump chain con `$ref` e con
  indirizzo, ciclo di jump, credenziale ereditata.
- Test sui piani: RDP con FreeRDP (fit, fixed, gateway), RDP con Windows App (contenuto `.rdp`) e `auto` senza client.
- In nessun argv e in nessuna env di nessun piano compare il valore di un segreto. Si verifica con un segreto
  sentinella nel MemoryVault e con `render()`.
- Askpass: test di integrazione sul socket con prompt simulati (password, passphrase, prompt sconosciuto,
  seconda richiesta → fallback, token errato → rifiutato).

---

## 8. Importer (`portolano-import`, WP5)

### 8.1 Trait generico

```rust
pub trait Importer: Send + Sync {
    fn id(&self) -> &'static str;                         // "royal"
    fn display_name(&self) -> &'static str;
    fn detect(&self, path: &Path, head: &[u8]) -> bool;  // estensione + magic/contenuto
    fn import(&self, input: &[u8], opts: &ImportOptions, prompter: &mut dyn SecretPrompter)
        -> Result<ImportOutcome, ImportError>;
}
pub struct ImportOptions { pub source_file_name: String, pub include_trash: bool }
pub struct ImportOutcome {
    pub root: Node,                                       // una cartella che contiene tutto l'import
    pub secrets: Vec<(SecretRef, Secret)>,                // da scrivere nel vault (lo fa la CLI)
    pub icons: Vec<PendingIcon>,                          // { placeholder_sha256, bytes, mime }: la CLI li salva con IconStore
    pub report: ImportReport,
}
pub trait SecretPrompter {
    fn ask(&mut self, req: &SecretRequest) -> Result<Option<Secret>, ImportError>; // None = salta
}
pub struct SecretRequest { pub node_path: String, pub username: Option<String>, pub field: SecretField }
pub enum SecretField { Password, Passphrase, PrivateKey, CustomProperty(String) }
pub struct NonInteractive;                                // risponde sempre None
pub struct Registry;                                      // with_defaults() → [royal]; detect(path, head)
```

Gli importer **non** accedono al vault né al filesystem, tranne leggere l'input che ricevono come bytes.

### 8.2 Report

```rust
pub struct ImportReport {
    pub importer: String, pub source_file: String, pub document_name: String, pub imported_at: DateTime<Utc>,
    pub counts: BTreeMap<String /*source_type*/, TypeCounts>,     // TypeCounts { source, imported, partial, unsupported, skipped }
    pub items: Vec<ReportItem>,     // { source_type, source_id, path, status, reasons: Vec<String> }
    pub unmapped_fields: BTreeMap<String /*type*/, BTreeMap<String /*field*/, usize>>,
    pub warnings: Vec<String>,
}
pub enum ItemStatus { Imported, Partial, Unsupported, Skipped }
```

- Va stampato come tabella leggibile e, con `--report <file>`, salvato come JSON.
- Contiene **solo** nomi, percorsi e motivi: niente segreti né ciphertext.
- Gli `items` includono solo gli elementi non `Imported`, a meno di `--verbose`.

### 8.3 Importer Royal

Il formato è descritto in [`royal-format.md`](royal-format.md), da leggere per intero. In sintesi operativa:

1. **Riconoscimento** (`sniff.rs`): BOM UTF-8 o `<` → XML; `1F 8B` → gzip (decomprimere con `flate2`, da aggiungere
   al crate); `PK` → zip (estrarre il primo `.xml`/`.rtsx` con `zip`). Radice attesa: `RTSZDocument`.
2. **Parsing** (`xml.rs`, `quick-xml`): ogni figlio della radice diventa
   `RawObject { type_name, fields: BTreeMap<String, RawField>, raw_xml: String }`, dove `RawField` è il testo,
   oppure l'XML interno se il campo ha figli. Nessun campo è obbligatorio, tranne `ID`.
3. **Albero**:
   - si usa `ParentID`;
   - l'ordinamento è per `PositionNr` numerico, con l'ordine nel file come criterio secondario;
   - `RoyalDocument` diventa la cartella `root` dell'outcome, con il nome del documento e i suoi campi in `extra`,
     escluso `PasswordHash`, che si scarta e si segnala nei warning come "documento protetto da password";
   - un `ParentID` che punta a un oggetto inesistente → il nodo va sotto `root` con un warning;
   - `RoyalTrash` e i suoi discendenti hanno stato `Skipped`; con `include_trash` finiscono nella cartella
     `Cestino (Royal)`.
4. **ID**: il `Node.id` è l'`ID` Royal, così il re-import è idempotente.
   `source = { system: "royal", source_type, source_id, file }`.
5. **Mappatura** (`map.rs`) secondo le §4–§8 di `royal-format.md`. Punti fissati:
   - `Name` vuoto → `"(senza nome)"`. `Created`/`Modified` si leggono nel formato `MM/dd/yyyy HH:mm:ss.fff`
     come UTC; se il parsing fallisce si usa l'ora attuale e il valore originale va in `extra`.
   - **Credenziali della connessione**:
     - `CredentialFromParent=True` → `Inherit`;
     - `CredentialMode` assente → `Inherit`;
     - `0` → `None`; `1` → `Prompt`; `2` → `Inline`; `3` → `ById(CredentialId)`; `4` → `ByName(CredentialName)`;
     - un valore sconosciuto dà `Inherit` e lo stato `Partial`;
     - un `ById` verso un ID non presente nel file resta così, con stato `Partial`
       ("riferimento a credenziale esterna al documento");
     - le cartelle seguono la stessa regola su `Folder.credential`.
   - **Segreti**: ogni campo cifrato presente e non vuoto (`Password`, `CredentialPassword`, e passphrase o chiave
     se presenti) genera un `SecretRequest`.
     - Se il prompter restituisce un valore, si crea un `SecretRef` e lo si aggiunge a `secrets`.
     - Se restituisce `None`, il campo resta vuoto e l'elemento passa a `Partial` con il motivo
       "password Royal cifrata non importata: impostala con `portolano secret set`".
   - `RoyalCredential`: `UserName` va in `username`.
     - Il dominio viene dal campo `Domain` se esiste, altrimenti da `DOMINIO\utente`.
     - I nomi nella forma `utente@dominio` restano come sono.
     - `PrivateKeyPath` → `PrivateKey::File`.
     - Una chiave incorporata genera un `SecretRequest(PrivateKey)`; senza valore, `Partial`.
   - `RoyalSSHConnection`:
     - `URI` → `host`, `Port` → `port`;
     - `SSHAllowAgentForwarding`, `SSHEnableX11Forwarding`, `SSHEnableTrustedX11Forwarding`, `SSHEnableCompression`,
       `KeepAliveInterval>0` → `SshSettings`;
     - campi del terminale → `extra`;
     - `IsTelnetConnection=True` o `IsSerialPortConnection=True` → `Unsupported`.
   - `RoyalSecureGateway` → connessione `ssh`. Le connessioni che la usano ricevono `jump_hosts: [{"$ref"}]`.
     I nomi dei campi di collegamento vanno scoperti dalle fixture; se restano incerti, il dato va in `extra`
     e l'elemento risulta `Partial`.
   - `RoyalRDSConnection`:
     - `URI`/`RDPPort` → `host`/`port`;
     - `DesktopWidth`/`DesktopHeight` a 0 o assenti → `fit`, altrimenti `fixed`;
     - `ColorDepth`, `ScaleFactor` (0 → None), `PhysicalAddress` → `wol_mac`;
     - i campi gateway e gli oggetti `RoyalRDSGateway` → `gateway`, con la stessa cautela delle Secure Gateway.
   - Icone:
     - `CustomImageName` non vuoto → `Icon::Royal{name}`;
     - `CustomImage` in base64 non vuoto → `PendingIcon` + `Icon::Custom`, con hash calcolato sui byte;
     - `CustomImageList` va in `extra`.
   - `Color` → `color`; `Favorite` → `favorite`; `Description` → `description`.
   - `Notes`: se è HTML, si converte in testo semplice per `notes` e l'originale va in `extra["royal.Notes"]`.
   - `CustomField1..10` non vuoti → `custom_properties` (Text); i campi custom protetti → Secret tramite prompt.
   - `IsConnectionTemplate=True` → nodo normale con tag `royal-template` e stato `Partial`
     ("i modelli non sono supportati").
   - **Qualsiasi** campo non mappato va in `extra["royal.<Campo>"]` come stringa e si conta in `unmapped_fields`.
   - Un tipo sconosciuto o non supportato (VNC, Web, File Transfer, Task, Dynamic Folder, …) diventa `Unsupported`
     con `raw` = XML originale e stato `Unsupported`.
6. **Fixture** (WP5): `crates/portolano-import/tests/fixtures/*.royal.xml`, sintetiche, con dati finti (§1.3).
   Servono almeno:
   - `minimal.royal.xml`, sul modello del doc B in `royal-format.md`;
   - `workspace.royal.xml`, sul modello del doc A: credenziali nei modi 2/3/4, trash, colori, terminale;
   - `full-synthetic.royal.xml`: tutti i casi `[ipo]` (gateway, secure gateway, note, custom field, icone,
     template, tipi non supportati, tipi sconosciuti, gzip);
   - le stesse fixture compresse in gzip, generate nel test.

   Le fixture sintetiche si dichiarano come tali in un commento XML, perché i nomi dei campi `[ipo]` sono ipotesi.
   `.gitignore`: va aggiunta l'eccezione `!crates/**/tests/fixtures/**`.
7. **Test di conteggio sui campioni reali** (`tests/samples.rs`, `#[ignore]`, attivato da `PORTOLANO_SAMPLES_DIR`):
   - per ogni `*.rtsz` / `*.rtsx` si contano direttamente nell'XML gli oggetti per tipo, esclusi quelli nel cestino;
   - si importa con `NonInteractive`;
   - si verifica che le cartelle importate siano pari a `RoyalFolder`, le connessioni a `RoyalSSHConnection` +
     `RoyalRDSConnection` (meno quelle telnet e seriali, che risultano `Unsupported`), e le credenziali
     a `RoyalCredential`;
   - si verifica che la somma degli stati nel report coincida con il numero di oggetti sorgente;
   - si stampano **solo** i conteggi.

### 8.4 Inserimento nel documento (lo fa la CLI, WP6)

`portolano import [--format royal] <file> [--into <path>] [--on-conflict fail|update|new-ids] [--prompt-secrets] [--include-trash] [--report <file.json>] [--dry-run]`

- **Formato**: si riconosce in automatico tramite `Registry::detect`; `--format` lo forza.
- **Destinazione**: `--into`, di default la radice. La cartella `root` dell'outcome viene inserita lì.
- **Conflitti** (un ID dell'outcome esiste già nel documento):
  - `fail` (default): errore, con l'elenco dei conflitti;
  - `update`: il nodo esistente viene sostituito dal nuovo, mantenendo posizione e padre attuali e i `SecretRef`
    già presenti quando il nuovo non ne porta;
  - `new-ids`: tutti gli ID vengono rigenerati e i `$ref` interni all'outcome rimappati.
- **Prompter**: con `--prompt-secrets` si chiede in modo interattivo, con input nascosto e invio vuoto per saltare,
  mostrando percorso e username. Senza l'opzione si usa `NonInteractive`.
- **Ordine delle operazioni**:
  1. si scrivono i segreti nel vault;
  2. si salvano le icone;
  3. si salva il documento;
  4. se il salvataggio fallisce, si cancellano i segreti appena scritti.
- `--dry-run`: non scrive nulla e mostra solo il report.

---

## 9. Configurazione (`config.rs`, WP2)

`config.toml` (tutti i campi sono opzionali):

```toml
default_document = "/Users/me/infra/portolano.json"
[vault]
backend = "keychain"          # keychain | file
file = "…/vault.json"
[terminal]
app = "terminal"              # terminal | iterm | ghostty | custom
command = ""                  # per custom, con {cmd}
[ssh]
program = "ssh"
[rdp]
client = "auto"               # auto | freerdp | windows-app
freerdp_program = ""          # override del percorso
cert = "tofu"                 # tofu | ignore | deny
```

API: `AppConfig::load_default() -> Result<AppConfig>`, `AppConfig::load(path)`, `save`.
Se il file non esiste si usano i default.

Ordine di precedenza, dal più forte: **flag CLI > variabile d'ambiente > config > default**.

---

## 10. CLI (`portolano-cli`, WP6)

### 10.1 Opzioni globali

- `--file <PATH>` (env `PORTOLANO_FILE`)
- `--vault keychain|file|memory` (env `PORTOLANO_VAULT`; `memory` è nascosto)
- `--vault-file <PATH>` (env `PORTOLANO_VAULT_FILE`)
- `--config <PATH>`
- `-v/--verbose` (ripetibile; i log vanno su stderr tramite `tracing`)
- `--json`, solo per i comandi di lettura

### 10.2 Comandi

| Comando | Comportamento |
|---|---|
| `init [--name N] [--force]` | crea il documento in `--file` o nella posizione di default |
| `tree [PATH] [--ids] [--depth N]` | albero; formato sotto |
| `list [PATH] [--type folder\|connection\|credential\|ssh\|rdp\|unsupported]` | tabella `TIPO  PERCORSO  RIEPILOGO` (ricorsiva) |
| `show <PATH> [--resolved]` | dettaglio del nodo. `--resolved` aggiunge la credenziale effettiva (origine, username, dominio, "password: presente/assente nel vault"). **Mai** il segreto |
| `add folder <PATH> [-p]` | |
| `add ssh <PATH> --host H [--port N] [--user U] [--cred SPEC] [--ask-password\|--password-stdin] [--key FILE] [--jump PATH\|ADDR]… [--agent-forwarding] [--x11] [--compression] [--keepalive S] [-o K=V]… [-p]` | `--user` senza `--cred` → `Inline` |
| `add rdp <PATH> --host H [--port N] [--user U] [--domain D] [--cred SPEC] [--ask-password\|--password-stdin] [--resolution fit\|fullscreen\|WxH] [--scale N] [--color-depth N] [--gateway HOST[:PORT]] [--gateway-cred SPEC] [--multimon] [--admin] [-p]` | |
| `add cred <PATH> [--user U] [--domain D] [--ask-password\|--password-stdin] [--key FILE \| --key-stdin] [--ask-passphrase] [-p]` | `--key-stdin` salva la chiave nel vault |
| opzioni comuni per `add` ed `edit` | `--description`, `--notes`, `--notes-file`, `--icon builtin:N\|royal:N\|file:PATH\|none`, `--color #RRGGBB`, `--favorite/--no-favorite`, `--tag T`…, `--prop K=V`…, `--secret-prop K` (prompt) |
| `edit <PATH> [--name N] [--host] [--port] [--cred SPEC] [--user/--domain] [--set K=JSON]… [--unset K]… [--untag T]… [--unprop K]…` | `--set` agisce su `settings` (il valore è JSON; se il parsing fallisce viene trattato come stringa) ed è validato dal Protocol |
| `move <PATH> <DEST\|/> [--position N \| --before PATH \| --after PATH]` | serve anche per riordinare (`DEST` = padre attuale) |
| `delete <PATH> [-r] [-y] [--force]` | una cartella non vuota richiede `-r`. Una credenziale referenziata viene rifiutata senza `--force` (con l'elenco dei riferimenti). Senza `-y` chiede conferma su TTY. Cancella dal vault i segreti del sottoalbero **dopo** aver salvato |
| `secret set <PATH> [--field password\|passphrase\|key\|prop:K] [--stdin]` | su `Credential`, o su `Connection` con `Inline` |
| `secret clear <PATH> [--field …]`, `secret status <PATH>` | `status` dice solo presente o assente |
| `connect <PATH> [--new-window] [--print] [--client freerdp\|windows-app] [--wait]` | `--print` mostra il piano (`render`) senza eseguirlo |
| `import …` | §8.4 |
| `validate` | core + protocolli + vault (`missing_secret`); exit 4 se ci sono errori |
| `config show`, `config set K V`, `config path` | |
| `askpass` | nascosto; la modalità askpass si attiva tramite variabile d'ambiente (§7.2) |

`SPEC` di `--cred`: `inherit` | `none` | `prompt` | `name:<NOME>` | `<PATH>` (→ `ById`).

Regole per le password:
- `--ask-password` usa un prompt nascosto con conferma, tramite `rpassword` su TTY;
- `--password-stdin` legge una riga da stdin;
- **non esiste** un'opzione `--password <valore>`.

Formato di `tree`:

```
Portolano
├── 📁 Clienti
│   ├── 📁 ACME
│   │   ├── [ssh] web01            admin@web01.example.com:22
│   │   └── [rdp] dc01             ACME\administrator@192.0.2.10
│   └── 🔑 Admin ACME              ACME\administrator
└── [?]  vnc-legacy                non supportato: RoyalVNCConnection
```

Con `--json` si stampa l'output strutturato senza segreti; per `tree` è il sottoalbero del modello.

### 10.3 Codici di uscita

| Codice | Significato |
|---|---|
| 0 | ok |
| 1 | errore generico |
| 2 | uso errato (clap) |
| 3 | non trovato o ambiguo |
| 4 | validazione |
| 5 | vault |
| 6 | errore di connessione prima del lancio |
| altro | `connect` restituisce l'exit code del client |

I messaggi d'errore sono in italiano, su stderr.

---

## 11. Test e qualità

### 11.1 Unit test minimi per WP

- **WP1**:
  - `insert`, `remove`, `move` (stesso padre in avanti e indietro, altro padre, radice), rifiuto dei cicli;
  - id univoci;
  - percorsi con escape, `@id` e prefisso, ambiguità, fallback case-insensitive;
  - risoluzione delle credenziali: catena di inherit su 3 livelli, `none` che interrompe la catena, `by_id`, `by_name`
    univoco/ambiguo/mancante, dangling;
  - `references_to` con un `$ref` annidato in `settings`;
  - `secret_refs_in`.
- **WP2**:
  - round-trip `save`/`load` byte-identico;
  - il JSON non contiene il valore di un segreto sentinella (i segreti stanno solo nel MemoryVault);
  - `schema_version` futuro → errore; migrazioni sintetiche;
  - salvataggio atomico (`.bak` creato, nessun `.tmp` residuo);
  - il lock impedisce un secondo `Store::open` concorrente;
  - `IconStore` dedup + gc;
  - config con default e precedenza.
- **WP3**:
  - `MemoryVault`;
  - `FileVault`: create, set, get, delete, riapertura, master password errata, file manomesso → errore;
    nessun segreto in chiaro nel file (grep del sentinella sui byte del file);
  - Keychain: test `#[ignore]` (`PORTOLANO_TEST_KEYCHAIN=1`) che crea e cancella una voce.
- **WP4**: §7.4. **WP5**: §8.3.6–7.
- **WP6**, test di integrazione con `assert_cmd`:
  - si usano directory temporanee, `--vault file` e `PORTOLANO_VAULT_PASSWORD`;
  - si copre ogni comando, inclusi gli errori;
  - si verifica che `connect --print` non contenga mai il segreto;
  - si verifica che `import` di una fixture produca i conteggi attesi.

### 11.2 Quality gate (dopo ogni merge)

`cargo fmt --all --check` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo test --workspace`

Inoltre `grep` sul repo per eventuali segreti sentinella o host reali: nessun IP fuori dagli intervalli RFC 5737 nei test.

### 11.3 Standard di codice (vincolanti per ogni WP)

Il codice deve leggersi come scritto da uno sviluppatore Rust esperto e attento, non generato in serie.

**Struttura**
- Moduli piccoli e coesi, ognuno con una sola responsabilità. Un file oltre ~400 righe va diviso; una funzione
  oltre ~50 righe va motivata o spezzata.
- Nessuna astrazione speculativa: niente trait, generici o layer che non servono a un caso d'uso della SPEC.
  Fanno eccezione i punti di estensione previsti (`Protocol`, `Importer`, `Vault`).
- Niente duplicazione copia-e-incolla: la logica ripetuta si fattorizza, ma solo dopo il secondo uso reale.
- Le dipendenze vanno dall'esterno verso il core, mai il contrario (§2).

**Stile**
- Rust idiomatico: `Result` con `?`, iteratori dove sono più chiari, ownership pensata (niente `.clone()`
  per far tacere il borrow checker).
- Nessun `unwrap()` / `expect()` / `panic!` nel codice non di test. Unica eccezione: invarianti dimostrabili, con un
  commento che spiega perché non può fallire.
- Errori tipizzati (`thiserror`) e con contesto: il messaggio dice *cosa* è fallito e *su quale* oggetto
  (id/percorso, mai segreti).
- Nomi espliciti e coerenti con il dominio della SPEC (Node, Folder, CredentialSpec, …). Niente abbreviazioni
  criptiche e niente `data`, `info`, `helper`, `manager` generici.
- `#[allow(...)]` solo con un commento di motivazione sulla riga sopra.

**Commenti e documentazione**
- Doc comment (`///`) su ogni elemento pubblico, con lo scopo e, dove serve, gli errori possibili.
- I commenti interni spiegano il *perché* (vincoli, scelte, riferimenti alla SPEC), mai il *cosa*.
  Niente commenti che parafrasano il codice.
- Niente codice commentato, niente `TODO` / `FIXME` orfani: i lavori rimandati vanno in `docs/progress.md`.
- Niente emoji nel codice e nei commenti. Le icone dell'output di `tree` (§10.2) sono l'unica eccezione e
  stanno in costanti.

**Test**
- Nome che descrive il comportamento (`move_into_own_descendant_is_rejected`), struttura arrange / act / assert.
- Un comportamento per test; fixture e builder condivisi in un modulo `testutil` del crate, non duplicati.
- Si testa il comportamento osservabile, non i dettagli interni.

**Commit**
- Piccoli e logici, ognuno compila e passa i test del crate. Messaggi in stile Conventional Commits, in inglese:
  `feat(core): add tree move with cycle detection`.
- Niente commit "wip", "fix", "update" generici e niente file non correlati al WP.

**Dubbi**: se la SPEC è ambigua, incompleta o sembra sbagliata, **non si indovina**. Il sottoagente si ferma su quel
punto e lo riporta al PM come domanda precisa, con le opzioni e la propria raccomandazione. Il PM decide solo se la
SPEC copre il caso; altrimenti chiede all'utente. La risposta va in `docs/DECISIONS.md`.

### 11.4 Smoke test (`scripts/smoke.sh`, WP7)

Lo script bash usa `set -euo pipefail`, una `mktemp -d` e il vault su file con la password di test. Esegue:
1. `init`;
2. `add folder -p Clienti/ACME`;
3. `add cred` (password da stdin);
4. `add ssh` con `--cred`;
5. `add ssh` con `--jump`;
6. `add rdp` con gateway;
7. `tree`, `list`, `show --resolved`;
8. `move` e riordino;
9. `edit --set`;
10. `validate`;
11. `connect --print` per ssh e rdp, con assert che il segreto non compaia;
12. `import` delle fixture con `--report`;
13. `delete -r`;
14. `secret status`.

Termina con `SMOKE OK`.

---

## 12. Pacchetti di lavoro (per i sottoagenti)

| WP | Contenuto | File (proprietà esclusiva) | Dipende da | Ondata |
|---|---|---|---|---|
| WP0 | Scheletro del workspace, tutte le dipendenze, **tutti i tipi del §3 e §5.1**, `error.rs`, stub `todo!()` con le firme di §4–§10, `.gitignore` aggiornato. Deve compilare (`cargo check --workspace`). | root `Cargo.toml`, tutti i `Cargo.toml` dei crate, `model.rs`, `secret.rs`, `error.rs`, `lib.rs` di tutti i crate, gli stub di ogni file | – | 0 |
| WP1 | Albero, percorsi, risoluzione, validazione | `tree.rs`, `path.rs`, `resolve.rs`, `validate.rs` | WP0 | 1 |
| WP2 | Store, migrazioni, icone, config | `store.rs`, `migrate.rs`, `icons.rs`, `config.rs` | WP0 | 1 |
| WP3 | Vault (3 backend) | `vault/*` | WP0 | 1 |
| WP4 | Connettori SSH/RDP, askpass, exec, terminali | `crates/portolano-connectors/**` | WP0 (usa le API WP1 tramite le firme; nei test costruisce documenti a mano) | 1 |
| WP5 | Importer generico + Royal + fixture | `crates/portolano-import/**` | WP0 | 1 |
| WP6 | CLI completa + test di integrazione | `crates/portolano-cli/**` | WP1–WP5 integrati | 2 |
| WP7 | `README.md`, `scripts/smoke.sh`, `docs/progress.md`, revisione di sicurezza (grep segreti, revisione argv/env/log) | quelli indicati | WP6 | 3 |

Le API di WP1, WP2 e WP3 sono fissate in questa specifica. WP4 e WP5 possono quindi lavorare in parallelo usando le
firme anche prima che le implementazioni siano unite. Nei propri test devono evitare chiamate ai `todo!()` altrui,
oppure attendere il merge dell'ondata e rieseguirli. Il PM integra l'ondata 1 nell'ordine WP1 → WP2 → WP3 → WP4 → WP5,
eseguendo i quality gate dopo ogni merge.

---

## 13. Definition of Done (app funzionante)

- [ ] Tutti i quality gate passano su un checkout pulito.
- [ ] `scripts/smoke.sh` termina con `SMOKE OK`.
- [ ] `cargo install --path crates/portolano-cli` installa `portolano`, e `portolano --help` elenca i comandi del §10.
- [ ] Il test di conteggio sui campioni reali passa (`PORTOLANO_SAMPLES_DIR=samples cargo test -- --ignored samples`).
- [ ] L'import dei due documenti in `samples/` riesce, con report: i segreti cifrati risultano `Partial` e i conteggi
      coincidono.
- [ ] Con il Keychain reale: `add cred --ask-password`, poi `secret status` → presente, poi `delete` → la voce
      sparisce dal portachiavi.
- [ ] `connect --print` di SSH e RDP mostra piani corretti; con FreeRDP installato, `connect` di una connessione RDP
      con credenziale salvata non chiede la password (**verifica manuale dell'utente** su un host reale).
- [ ] `connect` di una SSH con password salvata entra senza prompt (**verifica manuale dell'utente**).
- [ ] Il README contiene requisiti, build, posizione dei file, uso di ogni comando, sicurezza, limiti noti.
- [ ] `docs/progress.md` contiene un riepilogo di cosa funziona, cosa no e le ipotesi `[ipo]` ancora aperte.

## 14. Fuori scope (da non implementare ora)

UI Tauri, terminale integrato, RDP integrato (IronRDP), Wake-on-LAN, VNC/Web/SFTP, sincronizzazione o condivisione
di documenti, decrittaggio dei segreti Royal, Windows come piattaforma di test (deve solo compilare, se possibile).
