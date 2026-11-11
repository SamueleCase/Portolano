# Portolano

Portolano è un gestore di connessioni remote **SSH** e **RDP** per macOS, ispirato a Royal TSX. Le connessioni,
le cartelle e le credenziali stanno in un albero salvato in un file JSON leggibile (adatto anche a un repo git);
le password stanno solo nel **Keychain di macOS** o in un **vault su file cifrato**, mai nel documento.

Questa versione è a riga di comando (`portolano`):

- gestisce l'albero: cartelle, connessioni SSH e RDP, credenziali riusabili ed ereditate dalle cartelle;
- apre le connessioni con l'OpenSSH di sistema e con FreeRDP 3; è progettato per passare le password salvate
  senza richiederle e senza esporle in argomenti, variabili d'ambiente o file (il login automatico su host reali
  è ancora da verificare, vedi [limiti noti](#limiti-noti));
- importa i documenti di Royal TS/TSX (`.rtsz`, `.rtsx`), con un report di cosa è stato importato.

L'interfaccia grafica (Tauri) è prevista in una fase successiva; il core è già una libreria riusabile.

## Indice

- [Requisiti](#requisiti)
- [Build e installazione](#build-e-installazione)
- [Posizione dei file](#posizione-dei-file)
- [Concetti](#concetti)
- [Uso](#uso)
- [Configurazione](#configurazione)
- [Sicurezza](#sicurezza)
- [Limiti noti](#limiti-noti)
- [Struttura del progetto](#struttura-del-progetto)
- [Test](#test)

## Requisiti

- **macOS**: è l'unica piattaforma verificata. Il codice prevede anche Linux (portachiavi tramite Secret Service,
  terminale `x-terminal-emulator`), ma non è provato.
- **Rust stable** (≥ 1.99) installato con `rustup`. Il formula `rustup` di Homebrew è *keg-only*: non viene
  collegato in `/opt/homebrew/bin`, quindi la sua cartella va aggiunta al `PATH` (anche in `~/.zshrc`, per le
  sessioni successive), insieme a `~/.cargo/bin`, prima di usarlo:

  ```sh
  brew install rustup
  export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"
  rustup default stable
  ```

- **OpenSSH** (`ssh`, `ssh-add`): è quello di sistema di macOS.
- **FreeRDP 3**, consigliato per RDP: è l'unico client a cui Portolano passa la password salvata (login
  automatico ancora da verificare su un host reale).

  ```sh
  brew install freerdp
  ```

  Portolano cerca, in quest'ordine, `sdl-freerdp3`, `sdl-freerdp`, `xfreerdp3`, `xfreerdp` nel `PATH`, poi in
  `/opt/homebrew/bin` e `/usr/local/bin`.
- **Windows App** (Mac App Store, già *Microsoft Remote Desktop*): alternativa a FreeRDP, **senza autologon**:
  la password la chiede l'app a ogni connessione.

## Build e installazione

```sh
git clone <url-del-repo> Portolano
cd Portolano
cargo install --path crates/portolano-cli
```

Il binario `portolano` finisce in `~/.cargo/bin`, che deve essere nel `PATH` (vedi sopra). Verifica:

```sh
portolano --help
portolano --version
```

Per aggiornare, ripeti `cargo install --path crates/portolano-cli` dopo un `git pull`.

## Posizione dei file

Su macOS la cartella dei dati è `~/Library/Application Support/dev.portolano.Portolano/`.

| File | Posizione predefinita | Come cambiarla |
|---|---|---|
| Documento | `<dati>/portolano.json` | `--file`, `PORTOLANO_FILE`, `default_document` in `config.toml` |
| Configurazione | `<dati>/config.toml` | `--config` |
| Vault su file (solo backend `file`) | `<dati>/vault.json` | `--vault-file`, `PORTOLANO_VAULT_FILE`, `[vault] file` |
| Keychain (backend `keychain`) | portachiavi di login, servizio `dev.portolano`, un elemento per segreto | – |
| Icone personalizzate | `<cartella del documento>/<nome del file del documento senza estensione>.icons/<sha256>.<ext>` (es. `portolano.icons/`) | seguono il documento |
| Copia di sicurezza | `<documento>.bak`: la versione precedente, riscritta a ogni salvataggio | – |
| Lock | `<documento>.lock` e `vault.json.lock`: file vuoti usati per coordinare più processi; si possono ignorare | – |
| File temporanei e log dei client | `~/Library/Caches/dev.portolano.Portolano/tmp/` (cartella `0700`): i `.rdp` per Windows App e i log di FreeRDP (`0600`) | – |
| Socket askpass | `$TMPDIR/portolano-askpass-*/askpass.sock`, solo durante una sessione `ssh` | – |

I file temporanei e i log più vecchi di 10 minuti si cancellano al `connect` successivo. Il percorso del log di
FreeRDP viene stampato su stderr dopo l'avvio (`log del client: …`).

Il caso tipico per il documento è un file in un repo git: imposta `default_document` in `config.toml` oppure
`PORTOLANO_FILE`.

## Concetti

**Albero.** Il documento contiene cartelle, connessioni (`ssh`, `rdp`), credenziali e nodi *non supportati*
(oggetti importati da Royal che Portolano conserva ma non sa aprire). L'ordine dei figli è manuale. La profondità
massima è di 50 livelli.

**Percorsi.** Ogni comando indica i nodi con un percorso:

- segmenti separati da `/`, con lo `/` iniziale facoltativo: `Clienti/ACME/web01` = `/Clienti/ACME/web01`;
- `/` da solo è la radice (vale per `tree`, `list` e come destinazione di `move`);
- nei nomi, `\/` è uno slash letterale e `\\` un backslash: `Rete/LAN\/DMZ`;
- `@<id>` indirizza un nodo per id: l'UUID completo o un prefisso esadecimale di almeno 6 cifre (`@d6045d7d`);
  `tree --ids` e `show` mostrano gli id;
- il confronto è esatto; se non trova nulla riprova senza distinguere maiuscole e minuscole. Se più nodi
  corrispondono il comando esce con codice 3 ed elenca i candidati.

**Credenziali.** Una connessione o una cartella indica la propria credenziale con una *SPEC* (opzione `--cred`):

| SPEC | Significato |
|---|---|
| `inherit` | eredita dalla cartella padre (predefinito); alla radice: nessuna credenziale |
| `none` | nessuna credenziale; interrompe l'ereditarietà |
| `prompt` | chiede utente e password a ogni `connect` |
| `name:<NOME>` | la credenziale con quel nome, cercata in tutto il documento al momento della connessione |
| `<PERCORSO>` | una credenziale precisa, collegata per id (sopravvive a rinomina e spostamento) |

Con `--user` e senza `--cred`, `add ssh` e `add rdp` creano una credenziale *inline* propria della connessione.

## Uso

### Opzioni globali

Valgono per ogni comando, prima o dopo il nome del comando.

| Opzione | Variabile d'ambiente | Effetto |
|---|---|---|
| `--file <PATH>` | `PORTOLANO_FILE` | documento da usare |
| `--vault keychain\|file` | `PORTOLANO_VAULT` | backend dei segreti |
| `--vault-file <PATH>` | `PORTOLANO_VAULT_FILE` | file del vault; da solo implica `--vault file` (con `--vault keychain` è un errore d'uso) |
| `--config <PATH>` | – | file di configurazione |
| `-v`, `--verbose` | – | log su stderr; ripetibile (`-vv`, `-vvv`) |
| `--json` | – | output JSON, solo per i comandi di lettura (`tree`, `list`, `show`, `validate`, `secret status`, `config show`, `config path`); con un comando che modifica è un errore d'uso |

### `init`: creare il documento

```sh
portolano init                              # nella posizione predefinita, nome "Portolano"
portolano init --name Infrastruttura --file ~/infra/portolano.json
portolano init --force                      # sostituisce un documento esistente (resta il .bak)
```

Senza `--force`, se il file esiste il comando fallisce. Vedi i [limiti noti](#limiti-noti) per `--force` e il vault.

### `add`: aggiungere nodi

Opzioni comuni a tutti i tipi di `add` (e a `edit`): `-p`/`--parents` (crea le cartelle intermedie, solo `add`),
`--description`, `--notes` o `--notes-file FILE`, `--icon builtin:NOME|royal:NOME|file:PERCORSO|none`,
`--color '#RRGGBB'`, `--favorite`/`--no-favorite`, `--tag T` (ripetibile), `--prop CHIAVE=VALORE` (ripetibile),
`--secret-prop CHIAVE` (chiede il valore nascosto e lo salva nel vault). Un nome già usato da un fratello è
rifiutato.

**Cartelle**

```sh
portolano add folder -p Clienti/ACME
```

**Credenziali**

```sh
# password chiesta su terminale (input nascosto, con conferma)
portolano add cred "Clienti/ACME/Admin ACME" --user administrator --domain ACME --ask-password

# password da una riga di stdin (script, password manager)
op read "op://Infra/ACME/password" | portolano add cred "Clienti/ACME/Admin ACME" \
    --user administrator --domain ACME --password-stdin

# chiave privata su file, con la passphrase nel vault
portolano add cred Chiavi/deploy -p --user deploy --key ~/.ssh/id_ed25519_deploy --ask-passphrase

# chiave privata salvata nel vault (letta da stdin, poi il file si può eliminare)
portolano add cred Chiavi/ci -p --user ci --key-stdin < ./id_ed25519_ci
```

Non esiste un'opzione `--password <valore>`: una password sulla riga di comando finirebbe nella cronologia della
shell e nella lista dei processi. `--key` e `--key-stdin` si escludono; `--key-stdin` non si combina con
`--password-stdin` (usa `--ask-password`, oppure `secret set` dopo).

**Connessioni SSH**

```sh
# credenziale collegata per percorso
portolano add ssh Clienti/ACME/web01 --host web01.example.com --port 2222 \
    --cred "Clienti/ACME/Admin ACME"

# credenziale inline: utente e password propri della connessione
portolano add ssh Lab/vm1 -p --host 192.0.2.30 --user root --ask-password

# jump host: un'altra connessione ssh del documento oppure un indirizzo [utente@]host[:porta]
portolano add ssh Clienti/ACME/db01 --host db01.example.com --cred "Clienti/ACME/Admin ACME" \
    --jump Clienti/ACME/bastion --jump ops@edge.example.com:2200

# opzioni ssh
portolano add ssh Lab/vm2 --host 198.51.100.7 --user root --key ~/.ssh/id_lab \
    --agent-forwarding --compression --keepalive 30 -o StrictHostKeyChecking=accept-new
```

- `--jump` è ripetibile, nell'ordine dal primo salto all'ultimo. Un valore che corrisponde a un nodo diventa un
  riferimento (deve essere una connessione `ssh`, e i suoi jump vengono prima); un valore che non corrisponde a
  nessun nodo e non inizia con `/` o `@` è un indirizzo.
- `--x11` attiva l'inoltro X11 (`-X`); `-o CHIAVE=VALORE` è ripetibile e diventa `-o` di `ssh`.
- `--cred` non si combina con `--user`, `--ask-password` o `--password-stdin`: la password di una credenziale
  collegata si imposta su quella credenziale.

**Connessioni RDP**

```sh
portolano add rdp Clienti/ACME/dc01 --host 192.0.2.10 --cred "Clienti/ACME/Admin ACME" \
    --gateway rdg.example.com:443 --gateway-cred "Clienti/ACME/Admin ACME" --resolution 1920x1080

portolano add rdp Uffici/ts01 -p --host ts01.example.com --user alice --domain CORP --ask-password \
    --resolution fit --multimon
```

Opzioni: `--domain` (credenziale inline), `--resolution fit|fullscreen|LARGHEZZAxALTEZZA` (predefinito `fit`),
`--scale 100|140|180`, `--color-depth 15|16|24|32`, `--gateway HOST[:PORTA]`, `--gateway-cred SPEC` (richiede
`--gateway`), `--multimon`, `--admin`.

### `tree`, `list`, `show`: leggere

```sh
portolano tree
portolano tree Clienti --depth 2 --ids
portolano list --type ssh            # folder | connection | credential | ssh | rdp | unsupported
portolano list Clienti/ACME
portolano show Clienti/ACME/web01
portolano show --resolved Clienti/ACME/web01
portolano --json tree Clienti
```

```text
$ portolano tree
Portolano
└── 📁 Clienti
    └── 📁 ACME
        ├── 🔑 Admin ACME  ACME\administrator
        ├── [ssh] bastion  ops@bastion.example.com:22
        ├── [ssh] web01    administrator@web01.example.com:2222
        └── [rdp] dc01     ACME\administrator@192.0.2.10

$ portolano list --type ssh
TIPO  PERCORSO               RIEPILOGO
ssh   /Clienti/ACME/bastion  ops@bastion.example.com:22
ssh   /Clienti/ACME/web01    administrator@web01.example.com:2222
```

`show --resolved` aggiunge la credenziale effettiva: da dove arriva, utente, dominio e se la password è
*presente* o *assente* nel vault. Non mostra mai un segreto:

```text
Credenziale effettiva:
  Origine:  credenziale /Clienti/ACME/Admin ACME
  Utente:   administrator
  Dominio:  ACME
  Password: presente nel vault
```

### `edit`: modificare

```sh
portolano edit Clienti/ACME/web01 --name web01-prod --host web01-new.example.com --port 22 --tag prod
portolano edit Clienti/ACME/web01 --cred name:"Admin ACME"
portolano edit Lab/vm1 --user admin                     # utente della credenziale inline
portolano edit Uffici/ts01 --domain CORP2               # dominio della credenziale inline
portolano edit Clienti/ACME/web01 --set keepalive_s=30 --set compression=true
portolano edit Clienti/ACME/dc01 --set scale=140 --unset gateway
portolano edit Clienti/ACME/web01 --untag prod --unprop owner --description ""
```

`--set CHIAVE=JSON` agisce sulle impostazioni del protocollo (`settings`): il valore è JSON e, se non è JSON
valido, viene trattato come stringa. Il protocollo valida il risultato e rifiuta le chiavi sconosciute,
elencando quelle ammesse:

- **ssh**: `jump_hosts`, `agent_forwarding`, `x11`, `x11_trusted`, `compression`, `keepalive_s`, `options`;
- **rdp**: `resolution`, `scale`, `color_depth`, `gateway`, `multimon`, `admin`, `clipboard`, `wol_mac`,
  `client`.

Un valore vuoto (`--description ""`, `--user ""`, `--color ""`) cancella il campo.

`--host` e `--port` valgono per le connessioni. `--user` e `--domain` modificano i campi di una credenziale, o
della credenziale inline di una connessione; non si combinano con `--cred`. Come in `add`, una rinomina con
`--name` è rifiutata se un fratello ha già quel nome; lo stesso vale per `move` nella cartella di destinazione.

### `move`: spostare e riordinare

```sh
portolano move Clienti/ACME/dc01 Clienti/Windows       # in un'altra cartella, in coda
portolano move Clienti/ACME/web01 Clienti/ACME --position 0
portolano move Clienti/ACME/bastion Clienti/ACME --before Clienti/ACME/web01
portolano move Clienti/ACME/dc01 / --after Clienti     # nella radice
```

Per riordinare si indica come destinazione il padre attuale. Una cartella non si può spostare dentro sé stessa o
un suo discendente, e lo spostamento è rifiutato se nella destinazione c'è già un nodo con lo stesso nome.

### `delete`: eliminare

```sh
portolano delete Clienti/ACME/web01            # chiede conferma sul terminale
portolano delete Clienti/ACME -r -y            # una cartella non vuota richiede -r; -y salta la conferma
portolano delete "Clienti/ACME/Admin ACME" --force
```

- Se altri nodi, fuori da ciò che si elimina, vi fanno riferimento (credenziali collegate, jump host),
  l'eliminazione è rifiutata con l'elenco dei riferimenti. Con `--force` le connessioni che usavano una
  credenziale eliminata passano a `prompt`; gli altri riferimenti (es. jump host) restano e `validate` li segnala.
- Senza terminale e senza `-y` il comando fallisce invece di chiedere.
- I segreti del sottoalbero eliminato si cancellano dal vault **dopo** aver salvato il documento.

### `secret`: gestire le password

```sh
portolano secret set "Clienti/ACME/Admin ACME"                    # password, prompt nascosto con conferma
pass show acme/admin | portolano secret set "Clienti/ACME/Admin ACME" --stdin
portolano secret set Chiavi/deploy --field passphrase
portolano secret set Chiavi/ci --field key --stdin < ./id_ed25519_ci
portolano secret set Clienti/ACME/web01 --field prop:token        # proprietà segreta
portolano secret clear Lab/vm1
portolano secret status "Clienti/ACME/Admin ACME"
```

- `--field password|passphrase|key|prop:NOME` (predefinito `password`). La chiave privata si legge solo da stdin
  (tutto lo stdin); gli altri campi leggono una riga.
- `password` vale su una credenziale o su un nodo con credenziale inline; `passphrase` e `key` solo su una
  credenziale; `prop:NOME` su qualsiasi nodo (cartelle comprese). Su un nodo che non può contenere quel segreto
  il comando fallisce.
- Se il campo ha già un segreto, la voce del vault viene sovrascritta sul posto e il documento non cambia.
- `secret status` dice solo *presente* o *assente* nel vault per ogni campo.

### `connect`: aprire una connessione

```sh
portolano connect Clienti/ACME/web01                     # ssh nel terminale corrente
portolano connect Clienti/ACME/web01 --new-window        # in una nuova finestra del terminale
portolano connect Clienti/ACME/dc01                      # FreeRDP, si stacca subito
portolano connect Clienti/ACME/dc01 --wait               # attende la chiusura di FreeRDP
portolano connect Clienti/ACME/dc01 --client windows-app
portolano connect Clienti/ACME/web01 --print             # mostra il piano senza eseguirlo
```

`--print` mostra il comando che verrebbe eseguito; i segreti compaiono solo come riferimento `<vault:…>`. Per le
connessioni `db01` e `dc01` degli esempi di `add` (con `bastion` creata con `--user ops`):

```text
$ portolano connect --print Clienti/ACME/db01
/usr/bin/ssh -p 22 -l administrator -J ops@bastion.example.com:22,ops@edge.example.com:2200 -o 'NumberOfPasswordPrompts=1' -- db01.example.com
#   askpass (/Users/me/.cargo/bin/portolano): password di administrator@db01.example.com <- <vault:51c60753-9350-4b29-b081-c615f159bed9>

$ portolano connect --print Clienti/ACME/dc01
avviso: i messaggi d'errore di FreeRDP vengono salvati in un log nella cartella temporanea di Portolano
/opt/homebrew/bin/sdl-freerdp /v:192.0.2.10:3389 /u:administrator /d:ACME /cert:tofu /size:1920x1080 +clipboard /gateway:g:rdg.example.com:443 /from-stdin:force /log-level:WARN < <vault:51c60753-9350-4b29-b081-c615f159bed9> &
```

Come si comporta:

- **SSH**: usa l'`ssh` di sistema. Le password (della destinazione e dei jump host) e le passphrase arrivano a
  `ssh` tramite l'askpass di Portolano. Ogni segreto viene fornito una sola volta: se la password salvata è
  sbagliata Portolano non la ripete, e le richieste successive arrivano a te sul terminale. Anche la conferma
  della host key si chiede sul terminale. Il codice d'uscita di `connect` è quello di `ssh`.
- **Chiave nel vault**: viene aggiunta a `ssh-agent` per 5 minuti (`ssh-add -t 300 -`, chiave via stdin); serve un
  agent attivo (`SSH_AUTH_SOCK`).
- **RDP con FreeRDP**: la password passa via stdin (`/from-stdin:force`). Senza `--wait` FreeRDP si stacca dal
  terminale e i suoi messaggi d'errore vanno in un log. Se la credenziale del gateway coincide con quella della
  destinazione FreeRDP riusa le stesse credenziali; altrimenti la password del gateway la chiede FreeRDP.
- **RDP con Windows App**: Portolano genera un file `.rdp` senza password e lo apre con l'app, che chiede la
  password.
- **Client RDP**: `--client` > impostazione `client` della connessione > `[rdp] client` della configurazione;
  in `auto` si usa FreeRDP se c'è, altrimenti Windows App (con un avviso).
- **Credenziale `prompt`**: sul terminale `connect` chiede utente, password (e dominio per RDP); la password resta
  in memoria per quella sola connessione. Senza terminale li chiede il client.
- **Nuova finestra**: `--new-window` rilancia `portolano connect` nell'app di terminale configurata, con lo stesso
  documento, vault e configurazione; il vault si riapre nella nuova finestra (con il vault su file la master
  password si chiede lì: `PORTOLANO_VAULT_PASSWORD` non viene passata).

### `import`: importare da Royal TS/TSX

```sh
portolano import ~/Documents/Clienti.rtsz --dry-run                  # solo il report, nessuna scrittura
portolano import ~/Documents/Clienti.rtsz --into Import --prompt-secrets --report ~/import-report.json
portolano import ~/Documents/Clienti.rtsz --into Import --on-conflict update
```

- Il formato si riconosce dal contenuto (XML, gzip o zip); `--format royal` lo forza.
- `--into` indica una cartella esistente (predefinito: la radice); il documento Royal diventa una cartella con
  il suo nome.
- Le password di Royal sono cifrate con un algoritmo proprietario e **non vengono decifrate**. Con
  `--prompt-secrets` l'import chiede ogni password (input nascosto, invio vuoto per saltarla) mostrando percorso
  e utente, e la salva nel vault; richiede un terminale e non si combina con `--dry-run`. Senza, o se la salti,
  l'elemento risulta *parziale* e la password si imposta dopo con `portolano secret set`. Il ciphertext di Royal
  non viene conservato da nessuna parte.
- `--on-conflict fail|update|new-ids` (predefinito `fail`): gli id dei nodi importati sono quelli di Royal, quindi
  reimportare lo stesso file dà conflitti. `update` sostituisce i nodi esistenti mantenendone posizione e segreti;
  `new-ids` importa una copia con id nuovi.
- `--include-trash` importa anche il cestino di Royal nella cartella `Cestino (Royal)`.
- Il report viene stampato come tabella; con `--report FILE` è salvato anche in JSON. Contiene solo nomi,
  percorsi e motivi, mai segreti o ciphertext:

Estratto dell'output reale con la fixture di test `workspace.royal.xml` (le righe omesse sono segnate con `…`):

```text
$ portolano import crates/portolano-import/tests/fixtures/workspace.royal.xml --into Import
Import di 'workspace.royal.xml' (royal): documento 'Workspace di prova' in /Import/Workspace di prova
TIPO                ORIGINE  IMPORTATI  PARZIALI  NON SUPPORTATI  SALTATI
RoyalCredential     3        0          3         0               0
RoyalDocument       1        1          0         0               0
RoyalFolder         4        3          0         0               1
RoyalRDSConnection  5        3          1         0               1
RoyalSSHConnection  4        2          1         0               1
RoyalTrash          1        0          0         0               1

Elementi non importati completamente:
  [parziale] Workspace di prova/Credenziali/Admin ACME (RoyalCredential): password Royal cifrata non importata: impostala con `portolano secret set`
  …
  [saltato] Workspace di prova/Trash (RoyalTrash): cestino di Royal non importato (usa l'opzione per includerlo)
  [saltato] Workspace di prova/Trash/vecchio (RoyalSSHConnection): nel cestino di Royal
  …

Campi non mappati (conservati in extra):
  RoyalFolder: ColorName (1), CustomImageList (1), IsExpanded (1)
  …
avviso: documento protetto da password

Fatto: 14 nodi inseriti, 0 aggiornati, 0 segreti, 0 icone.
```

Gli oggetti Royal che Portolano non apre (VNC, Web, trasferimento file, cartelle dinamiche, Telnet, …) diventano
nodi *non supportati* (`[?]` in `tree`) che conservano l'XML originale, senza i campi cifrati.

### `validate`: controllare il documento

```sh
portolano validate
```

Controlla riferimenti rotti, credenziali per nome ambigue o mancanti, host vuoti, impostazioni dei protocolli e
segreti mancanti nel vault (avviso: il documento può essere usato da macchine con portachiavi diversi). Stampa
errori, avvisi e note (i nodi non supportati) ed esce con 4 se trova errori.

### `config`: configurazione

```sh
portolano config path                       # dove si trova config.toml
portolano config show
portolano config set rdp.client freerdp
portolano config set vault.backend file
portolano config set default_document ~/infra/portolano.json
```

Le chiavi sono `default_document` oppure `sezione.campo` (vedi [Configurazione](#configurazione)); i valori
vengono validati prima di scrivere il file.

### Codici d'uscita

| Codice | Significato |
|---|---|
| 0 | ok |
| 1 | errore generico (incluso: riferimenti senza `--force`, cartella non vuota senza `-r`, conferma mancante, documento modificato da un altro processo) |
| 2 | uso errato (opzioni sconosciute o incompatibili, `--json` con un comando che modifica) |
| 3 | nodo non trovato o percorso ambiguo |
| 4 | validazione: `validate` con errori, valori che renderebbero invalidi documento o configurazione |
| 5 | vault: master password errata, segreto non leggibile o non scrivibile |
| 6 | errore di connessione prima del lancio (client non trovato, nodo che non è una connessione, …) |
| altro | `connect` restituisce il codice d'uscita del client (`ssh`, FreeRDP con `--wait`) |

I messaggi d'errore sono in italiano, su stderr.

## Configurazione

`config.toml` è facoltativo; ogni chiave ha un default. Esempio completo:

```toml
default_document = "/Users/me/infra/portolano.json"

[vault]
backend = "keychain"          # keychain | file
# file = "/Users/me/infra/vault.json"   # solo per backend = "file"

[terminal]
app = "terminal"              # terminal | iterm | ghostty | custom
# command = "wezterm start -- {cmd}"    # per custom: {cmd} è il comando da eseguire

[ssh]
program = "ssh"               # nome nel PATH o percorso

[rdp]
client = "auto"               # auto | freerdp | windows-app
freerdp_program = ""          # percorso di FreeRDP; vuoto = ricerca automatica
cert = "tofu"                 # tofu | ignore | deny (opzione /cert: di FreeRDP)
```

Precedenza, dalla più forte: **opzione della riga di comando > variabile d'ambiente > `config.toml` > default**.
Ad esempio il documento è `--file`, altrimenti `PORTOLANO_FILE`, altrimenti `default_document`, altrimenti
`portolano.json` nella cartella dei dati. Un valore di primo livello sconosciuto (es. un refuso come
`default_documnt = "…"`) è un errore; una sezione sconosciuta (es. `[foo]`) viene conservata e ignorata.
`config show` mostra le sezioni `[ssh]` e `[rdp]` solo se sono presenti nel file.

## Sicurezza

**Dove stanno i segreti.** Password, passphrase, chiavi private e proprietà segrete stanno solo nel vault. Il
documento JSON contiene solo riferimenti opachi (UUID).

- **Keychain** (predefinito su macOS): un elemento per segreto nel portachiavi di login, servizio `dev.portolano`,
  account = riferimento. Protetto dal sistema e sbloccato con la sessione.
- **Vault su file** (`--vault file`): `vault.json` cifrato con XChaCha20-Poly1305, chiave derivata con Argon2id
  (64 MiB, 3 iterazioni) dalla master password. Utile senza portachiavi o per tenere vault separati; la master
  password si chiede con un prompt nascosto (due volte alla creazione).
- `PORTOLANO_VAULT_PASSWORD` fornisce la master password senza prompt: **solo per automazione e test**. Una
  variabile d'ambiente è visibile ai processi dello stesso utente e rischia di finire nella cronologia della
  shell. Portolano la toglie dall'ambiente di ogni processo che avvia (`ssh`, FreeRDP, `ssh-add`, terminale).

**Come arrivano ai client.**

- **SSH**: tramite `SSH_ASKPASS`. `connect` avvia un piccolo server su un socket Unix in una cartella privata
  (`0700`), con un token casuale monouso; `ssh` richiama `portolano` come askpass, che riceve la password dal
  server. Ogni segreto è servito una volta sola. A fine sessione socket e cartella si cancellano e i segreti in
  memoria si azzerano.
- **Chiavi nel vault**: `ssh-add` le legge da stdin; mai un file temporaneo con la chiave.
- **RDP**: FreeRDP riceve la password su stdin (`/from-stdin:force`); mai `/p:` negli argomenti. Il file `.rdp`
  per Windows App non contiene password.

Questi meccanismi sono coperti da test automatici (askpass con prompt simulati, piani di lancio con un segreto
sentinella); il login con host, agent e gateway reali è ancora da verificare (vedi [limiti noti](#limiti-noti)).

**Cosa non contiene mai un segreto:** il documento e il suo `.bak`, `config.toml`, il report d'import, l'output
della CLI (compreso `--print`, `--json` e `show --resolved`), i messaggi d'errore, i log (`-v`), gli argomenti e le
variabili d'ambiente dei processi avviati, i file temporanei. Le password non si passano mai come argomento:
`--ask-password` (prompt nascosto), `--password-stdin` o `secret set --stdin`.

I log di FreeRDP (`0600`, nella cartella privata della cache) possono contenere nomi di utenti e host.

**Prompt del portachiavi dopo ogni ricompilazione.** Con un binario non firmato (come quello di
`cargo install`), macOS può mostrare "portolano vuole accedere al portachiavi" dopo ogni ricompilazione, anche se
avevi scelto "Consenti sempre": la firma del binario è cambiata. È normale; scegli di nuovo "Consenti sempre".

## Limiti noti

- **`init --force`** sostituisce il documento (resta il `.bak`) ma non tocca il vault: i segreti del vecchio
  documento restano nel Keychain o nel vault su file.
- **`add ssh --key FILE`** salva la chiave nelle opzioni ssh della connessione (`IdentityFile`, `IdentitiesOnly`),
  non in una credenziale: `show --resolved` non la mostra come chiave. Per una chiave riusabile usa
  `add cred --key` e collega la credenziale con `--cred`.
- **`import --verbose`** non esiste: il report elenca solo gli elementi non importati completamente.
- **Credenziali inline dentro le impostazioni** (es. la credenziale del gateway di una connessione RDP importata):
  `secret set` e `secret clear` non le raggiungono, e `import --on-conflict update` non ne conserva i segreti.
- **`name:`** cerca la credenziale in tutto il documento: un import che porta un'altra credenziale con lo stesso
  nome rende ambiguo il riferimento (`validate` lo segnala). Il collegamento per percorso (`--cred <PERCORSO>`)
  non ha questo problema.
- Un nodo che si chiama esattamente come un riferimento per id (`@` seguito da 6 o più cifre esadecimali) è
  raggiungibile solo per id.
- **Profondità massima dell'albero: 50 livelli.** Oltre, `add`, `move` e l'import rifiutano o agganciano alla
  radice.
- I **log dei client** e i file `.rdp` restano nella cartella temporanea finché un `connect` successivo li trova
  più vecchi di 10 minuti.
- **Windows App** non fa autologon: la password la chiede l'app.
- **Gateway RDP**: con una credenziale diversa da quella della destinazione la password del gateway la chiede
  FreeRDP.
- **Login automatico non ancora provato su host reali**: che `connect` entri senza chiedere la password salvata,
  in SSH (askpass) e in RDP (FreeRDP con `/from-stdin:force`), è verificato solo sui piani di lancio e con test
  simulati. Manca la verifica manuale su host reali (Definition of Done, SPEC §13).
- **Da verificare dal vivo**: il testo del prompt della passphrase di `ssh-add` per le chiavi salvate nel vault, e
  il riuso delle credenziali sul gateway RDP insieme a `/from-stdin:force`. Sono implementati secondo la
  documentazione e i test, ma non ancora provati su un agent e un gateway reali. Da provare anche come FreeRDP
  chiede una password che Portolano non ha (connessione senza password salvata, gateway con credenziale
  diversa) quando si stacca dal terminale: senza `--wait` il suo stdin è vuoto o già chiuso; con `--wait` e senza
  password salvata FreeRDP resta collegato al terminale.
- **Royal**: alcuni campi (gateway, Secure Gateway, campi personalizzati protetti, chiavi incorporate) sono
  interpretati per ipotesi; gli elementi ricostruiti da questi campi risultano *parziali* nel report e conservano
  l'originale. Controllali dopo l'import.
- **Fuori da questa versione**: interfaccia grafica, terminale integrato, RDP integrato nell'app, Wake-on-LAN (il
  MAC si conserva soltanto), VNC/Web/SFTP, sincronizzazione o condivisione dei documenti, decrittaggio dei
  segreti di Royal. Windows non è una piattaforma verificata.

## Struttura del progetto

```
crates/
  portolano-core/        modello dati, albero e percorsi, risoluzione delle credenziali, validazione,
                         salvataggio atomico e migrazioni, icone, configurazione, vault (Keychain, file, memoria)
  portolano-connectors/  protocolli SSH e RDP: piani di lancio puri, esecuzione, askpass, terminali
  portolano-import/      trait Importer, report e importer Royal TS/TSX
  portolano-ops/         operazioni complete riusabili da CLI e futura app (add/edit con segreti, delete,
                         import con conflitti, validate, secret, connect), senza I/O su terminale
  portolano-cli/         il binario `portolano`: parsing, prompt e output
docs/                    SPEC.md (specifica), DECISIONS.md (decisioni), royal-format.md, progress.md
scripts/smoke.sh         test end-to-end
```

Le dipendenze vanno verso il core: `cli → {ops, core, connectors, import}`,
`ops → {core, connectors, import}`, `connectors → core`, `import → core`. Il core non conosce SSH, RDP o Royal.

## Test

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
scripts/smoke.sh
```

`scripts/smoke.sh` compila la CLI ed esegue i passi dello smoke test della specifica (SPEC §11.4), più
`secret clear` e `config path/show/set`, su un documento, una configurazione e un vault su file in una cartella
temporanea, con la master password di test. Cerca il segreto di prova nell'output di ogni comando e in tutti i
file prodotti. Non tocca i tuoi file né il Keychain e non avvia
client reali: `ssh`, FreeRDP, `ssh-add`, `osascript` e `open` sono sostituiti da stub. Termina con `SMOKE OK`.
Per provare un binario già compilato: `PORTOLANO_BIN=~/.cargo/bin/portolano scripts/smoke.sh`.

Test che richiedono risorse reali, esclusi di default:

```sh
# Keychain reale: crea e cancella un elemento di prova (può mostrare il prompt del portachiavi)
PORTOLANO_TEST_KEYCHAIN=1 cargo test -p portolano-core -- --ignored keychain

# conteggi sull'import di documenti Royal reali (stampa solo conteggi)
PORTOLANO_SAMPLES_DIR=samples cargo test -p portolano-import -- --ignored samples
```
