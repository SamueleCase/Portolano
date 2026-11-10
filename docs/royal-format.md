# Formato dei documenti Royal TS/TSX e mappatura sul modello Portolano

> FASE 0 – analisi. Basata su due documenti reali creati con **Royal TSX 6.3 (macOS)**.
> In questo file **non** compaiono host, IP, utenti o segreti reali: per i campi sensibili
> si riportano solo forma, lunghezza e presenza.
>
> Legenda affidabilità: **[oss]** osservato nei campioni · **[ded]** dedotto dai campioni ·
> **[ipo]** ipotesi da verificare con campioni aggiuntivi.

## 1. Campioni analizzati

| Doc | Note | Oggetti |
|-----|------|---------|
| A | Workspace personale, **protetto da password documento** (`PasswordHash` presente), creato a mano con Royal TSX | 1 Document, 1 Trash, 4 Folder, 4 SSH, 6 RDS (RDP), 7 Credential |
| B | Senza segreti, campi minimi e timestamp tutti uguali: con ogni probabilità **generato da uno script** | 1 Document, 5 Folder, 1 SSH, 1 RDS |

La cartella `samples/` del repo non esiste ancora; i file sono stati letti in sola lettura da una copia
temporanea. Non ci sono icone custom, note, proprietà custom, gateway RDP né jump host: per questi
aspetti la mappatura è **[ipo]** finché non arriva un documento di test più ricco (vedi §9).

## 2. Contenitore e codifica

- **[oss]** Entrambi i `.rtsz` sono **XML UTF-8 in chiaro con BOM** (`EF BB BF`), non compressi.
- **[ipo]** Le versioni storiche di Royal TS usavano `.rtsx` per l'XML in chiaro e `.rtsz` per la variante
  compressa. L'importer quindi **non si fida dell'estensione** e riconosce il formato dai magic byte:
  - `EF BB BF` / `<` → XML
  - `1F 8B` → gzip, da decomprimere e poi trattare come XML
  - `50 4B` (`PK`) → zip, si cerca l'XML all'interno
- **[oss]** Radice: `<RTSZDocument>` senza attributi.

## 3. Struttura: lista piatta + `ParentID`

- **[oss]** Tutti gli oggetti sono **figli diretti** di `<RTSZDocument>`. L'albero si ricostruisce con
  `ID` e `ParentID`, che sono UUID minuscoli.
- **[oss]** `RoyalDocument` è la radice logica (senza `ParentID`). Cartelle, `RoyalTrash` e oggetti di primo
  livello hanno `ParentID` = ID del documento.
- **[oss]** Le **credenziali vivono nell'albero** insieme alle connessioni, dentro le cartelle.
- **[oss]** L'ordine dei fratelli è dato da `PositionNr`: è una **stringa da ordinare come numero**
  (`"10"` viene dopo `"8"`), con passo 2 nel doc A e passo 1 nel doc B, e a volte manca.
  Regola: si ordina per `PositionNr` numerico, poi per ordine di apparizione nell'XML.
- **[oss]** Profondità osservata fino a 3 livelli; il formato non pone limiti.
- **[oss]** Ogni campo è un elemento figlio (`<Name>…</Name>`), mai un attributo. I campi con valore di
  default sono **omessi**: nel doc B mancano quasi tutti. L'importer deve applicare i default di Royal
  e non dare nessun campo per scontato.
- **[oss]** Booleani `"True"`/`"False"`. Date nel formato `MM/dd/yyyy HH:mm:ss.fff` senza fuso orario
  (**[ipo]** UTC). Il valore originale si conserva comunque.

## 4. Tipi di oggetto

### 4.1 Osservati

| Tipo Royal | Campi osservati (oltre ai comuni) | Mappa su |
|---|---|---|
| `RoyalDocument` | `DocumentType`=`Workspace`, `PasswordHash`, `SaveOption`, `AutoMergeOption`, `LockdownFeaturesInUse`, `DeletedObjectHistory`, `IsExpanded` | `Document` (metadati) + `extra` |
| `RoyalTrash` | solo comuni | non è un nodo: gli oggetti nel cestino vengono saltati e **segnalati** (opzione `--include-trash`) |
| `RoyalFolder` | `Description`, `IsExpanded` | `Folder` |
| `RoyalSSHConnection` | `URI`, `Port`, `ConnectionType`=`ssh;SSH`, `IsTelnetConnection`, `IsSerialPortConnection`, credenziali, SSH (`SSHAllowAgentForwarding`, `SSHEnableX11Forwarding`, `SSHEnableTrustedX11Forwarding`, `SSHEnableCompression`, `SSHDontAllocateAPseudoTerminal`, `KeepAliveInterval`, `WarnFingerprintMismatch`, `IPVersionOSX`), terminale (`Font*OSX`, `TerminalColumns/Rows`, `CursorAppearance`, `BlinkCursor`, …) | `Connection{protocol: ssh}` |
| `RoyalRDSConnection` | `URI`, `RDPPort`, credenziali, `DesktopWidth/Height` (0 = adatta alla finestra), `ColorDepth`, `ScaleFactor`, `AutomaticallyCalculateScaleFactor`, `SmartSizing`, `SmartReconnect`, `UseRetinaResolution`, `PhysicalAddress` (MAC per Wake-on-LAN) | `Connection{protocol: rdp}` |
| `RoyalCredential` | `UserName`, `Password` (cifrata), `PrivateKeyPath`, `PrivateKeyMode` | `Credential` |

**Campi comuni** **[oss]**: `ID`, `ParentID`, `Name`, `Description`, `PositionNr`, `Created`, `CreatedBy`,
`Modified`, `ModifiedBy`, `Favorite`, `IsConnectionTemplate`, `Color` (`#RRGGBB`), `ColorName`
(es. `Royal.Mariner`), `ColorFromParent`, `CustomImage`, `CustomImageName`, `CustomImageList`
(sempre `{}` nei campioni).

**Osservazioni**:
- `URI` contiene **solo l'host** (hostname o IPv4). La porta sta in `Port` (SSH) o `RDPPort` (RDP).
- Nell'SSH `ConnectionType` ha forma `<id>;<etichetta>`. Gli stessi oggetti `RoyalSSHConnection`
  coprono anche **Telnet** e **seriale** tramite `IsTelnetConnection` e `IsSerialPortConnection`.

### 4.2 Attesi ma non osservati **[ipo]**

Royal ha altri tipi da riconoscere almeno per nome, conservandoli come nodi `unsupported` con XML grezzo
e segnalandoli nel report:
`RoyalVNCConnection`, `RoyalWebConnection`, `RoyalFileTransferConnection` (SFTP/FTP/SCP),
`RoyalTeamViewerConnection`, `RoyalTerminalConnection`, `RoyalCommandTask`, `RoyalKeySequenceTask`,
`RoyalDynamicFolder`, `RoyalToDo`, `RoyalInformation`, e soprattutto:
- **`RoyalSecureGateway`**: è il modo in cui Royal realizza tunnel SSH e jump host. Diventa la catena
  `jump_hosts` della connessione.
- **`RoyalRDSGateway`**: oggetto Remote Desktop Gateway riutilizzabile. Diventa `rdp.gateway`.

Regola generale: un **tipo sconosciuto** non fa mai fallire l'import. Diventa un nodo `unsupported`
con `raw_xml` e mantiene la sua posizione nell'albero.

## 5. Credenziali e segreti

### 5.1 Come una connessione indica le credenziali **[ded]**

| `CredentialMode` | Significato | Campi usati | Mappa su `credential` |
|---|---|---|---|
| (assente) / `0` | nessuna **[ipo]** | – | `none` |
| `1` | **[ipo]** chiedi all'avvio | – | `prompt` |
| `2` | username/password inline | `CredentialUsername`, `CredentialPassword` (cifrata) | `inline{username, secret_ref}` |
| `3` | credenziale esistente **per ID** | `CredentialId` | `by_id{credential_id}` |
| `4` | credenziale esistente **per nome** | `CredentialName` (`CredentialId` può esserci come cache, anche non aggiornata) | `by_name{name}` (risolta all'avvio, come fa Royal) |

- `CredentialFromParent=True` → la credenziale si eredita dalla cartella padre (`inherit`).
  Anche le cartelle Royal possono avere credenziali, quindi il modello prevede `Folder.defaults.credential`.
- `CredentialAutologon`, `CredentialOmitDomain` → finiscono nelle opzioni della connessione.
- **[oss]** Nei campioni tutti i `CredentialId` puntano a un `RoyalCredential` dello stesso documento.
  **[ipo]** Possono anche puntare a oggetti di un altro documento (Royal supporta i riferimenti
  tra documenti). L'importer segnala i riferimenti che non riesce a risolvere.

### 5.2 Chiavi SSH **[ded]/[ipo]**

- `RoyalCredential.PrivateKeyPath` + `PrivateKeyMode=0` → chiave **su file** (percorso assoluto).
  **[ipo]** Un altro valore di `PrivateKeyMode` indica la chiave incorporata nel documento.
- `RoyalSSHConnection.CredentialKeyMode=0` **[ipo]**: la chiave viene dalla credenziale.
- Passphrase: non osservata **[ipo]**, sarà cifrata come le password.

### 5.3 Cosa è cifrato e come

- **[oss]** Sono cifrati `RoyalCredential.Password` e `CredentialPassword` inline: Base64 → **80 byte fissi**,
  qualunque sia la lunghezza della password. È coerente con un cifrario a blocchi da 16 byte, cioè
  AES con IV, padding a dimensione fissa e/o MAC.
  Il resto (host, utenti, percorsi delle chiavi, note) è **in chiaro**.
- **[oss]** Il doc A ha `RoyalDocument.PasswordHash` (Base64 → 192 byte): il documento è protetto da una
  password, che con ogni probabilità serve anche a ricavare la chiave dei campi cifrati.
- **[ipo]** Senza password documento Royal usa una chiave predefinita interna all'applicazione.
- Royal Apps **non pubblica** l'algoritmo. Per non aggirare protezioni, per la FASE 2a propongo
  questa strategia:
  1. **Default**: si importano tutta la struttura e gli username. Per ogni segreto cifrato l'importer
     **chiede la password in modo interattivo** (`--prompt-secrets`) e la salva nel vault.
     In alternativa marca la credenziale come *parziale* nel report.
     Nel doc A si tratta di **7 segreti** (6 credenziali + 1 inline): reinserirli è sostenibile.
  2. Il ciphertext Royal originale **non viene conservato** da nessuna parte (decisione D3 in `SPEC.md`).
  3. **Da decidere con te**: se Royal TSX offre un export ufficiale con password in chiaro (da verificare
     nel menu *File → Export*), l'importer può leggere anche quel formato. Implementare il decrittaggio
     del formato proprietario, anche con la tua password, richiederebbe invece di analizzare l'app:
     non lo faccio senza il tuo via libera esplicito.

## 6. Icone e immagini

- **[oss]** Royal TSX include una libreria di **869 icone vettoriali (PDF)** in
  `Royal TSX.app/Contents/Resources/IconLibrary/`, divise in due set: `Flat` (363) e `VMware Clarity` (506),
  organizzate in `<set>/<categoria>/<nome>.pdf`.
- **[ipo]** `CustomImageName` contiene il percorso logico nella libreria (es. `Flat/Hardware/Server`),
  `CustomImage` un'immagine custom incorporata in Base64, `CustomImageList` un dizionario di varianti.
  Nei campioni sono vuoti o `{}`.
- Mappatura:
  - `CustomImageName` → `Icon{kind: "royal", name: "<set>/<categoria>/<nome>"}`. Nella FASE 2 l'icona si
    risolve dalla libreria di Royal, se installata, o da una tabella di corrispondenza verso il nostro set.
    Le icone di Royal **non vengono copiate** nel nostro repo per motivi di licenza.
  - `CustomImage` → si decodifica, si salva in `icons/<sha256>.<ext>` accanto al documento e diventa
    `Icon{kind: "custom", sha256, mime}`.

## 7. Modello Portolano (bozza per la FASE 1)

```
Document   { schema_version, id, name, created, modified, nodes[], extra{}, import_source? }
Node (comune) {
  id (uuid; per gli import = ID Royal, così il re-import è idempotente),
  parent_id, position (int), name, description, notes, icon?, color?, favorite, tags[],
  custom_properties[] { name, value | secret_ref },
  created, modified,
  extra{}            // tutto ciò che non mappiamo: chiave = nome campo Royal, valore = stringa grezza
  source? { system: "royal", type: "RoyalSSHConnection", file, royal_id }
}
Folder     : Node { defaults { credential? } }
Connection : Node { protocol: "ssh" | "rdp" | …, host, port,
                    credential: CredentialSpec, options{ autologon, … }, settings (per protocollo) }
  ssh.settings { jump_hosts[] (ref a Connection), identity?, agent_forwarding, x11, compression,
                 keepalive_s, terminal{…} }
  rdp.settings { domain?, resolution{ mode: fit|fixed, width, height }, scale, color_depth,
                 smart_sizing, retina, gateway?{ host, port, credential: CredentialSpec }, wol_mac? }
Credential : Node { username, domain?, secret_ref?, private_key?{ path | secret_ref }, passphrase_ref? }
CredentialSpec = inherit | none | prompt | inline{username, domain?, secret_ref?} | by_id{id} | by_name{name}
Icon       = builtin{name} | royal{name} | custom{sha256, mime}
Unsupported: Node { royal_type, raw_xml }
```

`secret_ref` è una chiave opaca del vault (Keychain: service `portolano`, account `<uuid>`).
Nel file JSON non finisce **mai** un segreto.

## 8. Tabella di mappatura sintetica

| Royal | Portolano | Note |
|---|---|---|
| `ID` / `ParentID` / `PositionNr` | `id` / `parent_id` / `position` | position rinumerata 0..n |
| `Name`, `Description` | `name`, `description` | |
| `Notes` **[ipo]** | `notes` | HTML/RTF di Royal convertito in testo/Markdown; originale conservato in `extra` |
| `CustomField1..10`, `CustomProperties` **[ipo]** | `custom_properties[]` | i campi protetti vanno nel vault |
| `Color`, `ColorName`, `ColorFromParent` | `color` (+ inherit) | |
| `Favorite` | `favorite` | |
| `CustomImageName` / `CustomImage` | `icon` | §6 |
| `IsConnectionTemplate=True` | `extra` + voce nel report | modelli non supportati in FASE 1 |
| `URI` + `Port`/`RDPPort` | `host`, `port` | |
| `Credential*` | `credential` | §5 |
| campi del terminale `*OSX` | `ssh.settings.terminal` / `extra` | utili per il terminale della FASE 2 |
| `PhysicalAddress` | `rdp.settings.wol_mac` | |
| `Created/Modified(By)` | `created`, `modified` (+ `extra` con originali) | |
| `RoyalDocument.PasswordHash`, `SaveOption`, `DeletedObjectHistory`, `LockdownFeaturesInUse` | `Document.extra` | `PasswordHash` **non** viene salvato (inutile e sensibile) |
| qualsiasi campo non elencato | `extra[<nome>]` | conteggiato nel report come “campo non mappato” |

## 9. Cosa manca per chiudere le [ipo]

Serve un **documento di test con dati finti** (`samples/test-completo.rtsz`), creato in Royal TSX, che contenga:
- cartelle annidate su ≥ 4 livelli, una cartella con credenziale ereditata dai figli
- SSH con password, SSH con chiave su file + passphrase, SSH con chiave incorporata, SSH via Secure Gateway (jump host)
- RDP con dominio, risoluzione fissa, RD Gateway (oggetto riutilizzabile)
- una connessione per tipo non supportato (VNC, Web, SFTP)
- note, custom field, una proprietà custom protetta
- un'icona dalla libreria e un'icona custom (PNG)
- un oggetto nel cestino e una connessione-modello
- una copia **senza** password documento e una **con** password documento nota (finta)
