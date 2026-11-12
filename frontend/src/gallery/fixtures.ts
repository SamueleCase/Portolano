import type { MenuEntry } from "../components/ContextMenu";
import type { CredentialPreviewData } from "../components/credentialPreviewData";
import type { CustomIconOption } from "../components/IconPicker";
import type { KeyValueEntry } from "../components/KeyValueEditor";
import type { PickerNode } from "../components/NodePicker";
import type { WorkTab } from "../components/Tabs";
import type { ToastMessage } from "../components/Toast";

/**
 * Invented data for the gallery: hosts of RFC 5737 and `*.example.com`,
 * fake users (SPEC §1.3). Screen texts here are samples, not the texts of
 * the screens, which belong to their work packages.
 */

/** A name with control and bidi characters, as an untrusted document could carry. */
export const hostileName = `web01\u001b[2J\u0007${String.fromCodePoint(0x202e)}gpj.exe`;

export const sample = {
  documentTitle: "Il documento è vuoto",
  documentText:
    "Aggiungi la prima connessione oppure importa quelle che hai già in Royal TSX. Le password restano nel portachiavi di sistema.",
  newConnection: "Nuova connessione",
  importRoyal: "Importa da Royal TSX",
  commandCaption:
    "Comando che verrà lanciato. La password non compare: viene letta dal portachiavi all'avvio.",
  command:
    "ssh -p 22 -l deploy -J deploy@bastion.example.com:2222 -A -o ServerAliveInterval=30 -o StrictHostKeyChecking=accept-new -- web01.example.com",
  dropTitle: "Trascina qui il documento di Royal TSX",
  dropHint: "File .rtsz, .rtsx o XML, oppure",
  chooseFile: "Scegli file…",
  dialogTitle: "Imposta la password",
  dialogSubtitle: "Credenziali / admin · ACME",
  dialogText:
    "Verrà salvata nel vault quando salvi il nodo. Dopo il salvataggio non sarà più visibile, nemmeno qui.",
  password: "Password",
  confirm: "Conferma",
  cancel: "Annulla",
  set: "Imposta",
  editorTitle: "Modifica connessione SSH",
  name: "Nome",
  description: "Descrizione",
  notes: "Note",
  host: "Host",
  port: "Porta",
  keepalive: "Keepalive",
  portError: "La porta va da 1 a 65535",
  passwordsMatch: "Le due password coincidono",
  passwordsDiffer: "Le due password non coincidono",
  search: "Cerca per nome, host, tag…",
  depth: "Profondità colore",
  tags: "Tag",
  properties: "Proprietà custom",
  addProperty: "+ Proprietà",
  options: "Opzioni libere (chiave = valore)",
  addOption: "+ Opzione",
  optionError: "Opzione non ammessa: aggiungila al tuo ~/.ssh/config",
  toggles: ["Agent forwarding", "Compressione", "X11 forwarding", "X11 trusted"],
  x11Hint: "attivabile solo con X11 acceso",
  resolution: "Risoluzione",
  resolutions: ["Adatta alla finestra", "Schermo intero", "Fissa"],
  theme: "Tema",
  themes: ["Scuro", "Chiaro · in arrivo"],
  filters: ["Tutti", "SSH", "RDP", "Credenziali"],
  favorites: "Preferiti",
  reportFilters: ["Tutti", "Parziali", "Non supportati"],
  tooltip: "Connetti",
  tooltipShortcut: "⌘↩",
  tooltipLong: "Ereditata dalla cartella Produzione. La password è nel portachiavi.",
  connect: "Connetti",
  inherited: "Ereditata dalla cartella Produzione",
  inheritedOrigin: "dalla cartella che contiene il nodo",
  credentialName: "admin · ACME",
  domainUser: "ACME\\admin",
  passphrase: "Passphrase",
  token: "Token API",
  fromCredential: "Da una credenziale",
  validationPath: "Archivio / old-gw",
  validationMessage: "Il jump host fa riferimento a un nodo che non esiste più.",
  progress: "Password da reinserire",
  report: "Elementi letti",
  pickerTitle: "Scegli un jump host",
  pickerFilter: "Solo SSH",
  pickerNote: "web01 è escluso perché non può fare da salto a se stesso.",
  terminalTitle: "deploy@web01: ~",
  link: "https://www.example.com/runbook",
} as const;

export const buttonLabels = {
  primary: "Salva",
  secondary: "Annulla",
  toolbar: "+ Connessione",
  ghost: "Importa",
  add: "+ Proprietà",
  danger: "Elimina comunque",
  "danger-text": "Elimina",
  small: "Scegli…",
  large: "Nuova connessione",
  largeSecondary: "Importa da Royal TSX",
} as const;

export const workTabs: WorkTab[] = [
  { id: "detail", kind: "detail", title: "web01", dot: "detail", closable: false },
  {
    id: "validation",
    kind: "validation",
    title: "Validazione",
    dot: "validation",
    closable: true,
    problems: 4,
  },
  { id: "s1", kind: "ssh", title: "bastion", dot: "connecting", closable: true },
  { id: "s2", kind: "ssh", title: "web01", dot: "connected", closable: true },
  { id: "s3", kind: "ssh", title: "db02", dot: "failed", closable: true },
  { id: "s4", kind: "rdp", title: "win-app01", dot: "connected", closable: true },
  { id: "s5", kind: "external", title: "stg-win01", dot: "connected", closable: true },
];

export const steps = {
  wizard: [
    { label: "File", state: "done" },
    { label: "Opzioni", state: "current" },
    { label: "Password", state: "upcoming" },
    { label: "Report", state: "upcoming" },
  ],
  wizardSkipped: [
    { label: "File", state: "done" },
    { label: "Opzioni", state: "done" },
    { label: "Password", state: "skipped" },
    { label: "Report", state: "current" },
  ],
  editor: ["Generale", "Connessione"],
  editorOpen: [
    { label: "Generale", state: "current" },
    { label: "Connessione", state: "upcoming" },
  ],
} as const satisfies Record<string, readonly unknown[]>;

export const menuEntries: MenuEntry[] = [
  { id: "connect", label: "Connetti", shortcut: "⌘↩" },
  { id: "external", label: "Apri in terminale esterno" },
  { id: "command", label: "Mostra comando" },
  { separator: true },
  { id: "edit", label: "Modifica…", shortcut: "⌘E" },
  { id: "duplicate", label: "Duplica", shortcut: "⌘D" },
  { id: "favorite", label: "Preferito ★" },
  { separator: true },
  { id: "delete", label: "Elimina…", shortcut: "⌘⌫", destructive: true },
];

export const toasts: ToastMessage[] = [
  { id: "saved", kind: "success", text: "Password salvata nel portachiavi" },
  { id: "info", kind: "info", text: "Documento ricaricato" },
  {
    id: "refused",
    kind: "error",
    text: "db02: connessione rifiutata",
    action: { label: "Riprova", onAction: () => undefined },
  },
  {
    id: "deleted",
    kind: "undo",
    text: "web02 eliminato",
    action: { label: "Annulla", onAction: () => undefined },
  },
];

export const routeStops = {
  detail: [{ label: "bastion.example.com" }, { label: "web01.example.com", kind: "ssh" }],
  editor: [{ label: "bastion" }, { label: "198.51.100.40" }, { label: "web01" }],
  inherit: [{ label: "web01" }, { label: "cartella Produzione" }, { label: "credenziale deploy" }],
  session: ["bastion.example.com", "db02.example.com"],
} as const;

export const properties: KeyValueEntry[] = [
  { id: "env", key: "Ambiente", type: "text", value: "produzione" },
  { id: "token", key: "Token API", type: "secret", value: "", secret: "staged" },
];

export const sshOptions: KeyValueEntry[] = [
  { id: "strict", key: "StrictHostKeyChecking", type: "text", value: "accept-new" },
  { id: "proxy", key: "ProxyCommand", type: "text", value: "nc %h %p", error: sample.optionError },
];

export interface JumpHost {
  id: string;
  name: string;
  detail: string;
  source: "tree" | "address";
}

export const jumpHosts: JumpHost[] = [
  { id: "j1", name: "bastion", detail: "Produzione / bastion", source: "tree" },
  { id: "j2", name: "deploy@198.51.100.40:22", detail: "", source: "address" },
  { id: "j3", name: "gw-staging", detail: "Staging / gw-staging", source: "tree" },
];

export const validationRows = [
  {
    severity: "error",
    path: "Archivio / old-gw",
    message: "Il jump host fa riferimento a un nodo che non esiste più.",
  },
  {
    severity: "warning",
    path: "Staging / stg-win01",
    message: 'La credenziale "admin · ACME" non ha password: verrà chiesta a ogni avvio.',
  },
  {
    severity: "info",
    path: "vnc-legacy",
    message: "Nodo non supportato (VNC), conservato in sola lettura.",
  },
] as const;

export const reportRows = [
  {
    outcome: "partial",
    outcomeLabel: "Parziale",
    path: "Produzione / win-app01",
    reason: "Reindirizzamento delle stampanti non gestito, ignorato.",
  },
  {
    outcome: "unsupported",
    outcomeLabel: "Non supportato",
    path: "vnc-legacy",
    reason: "Tipo VNC: conservato in sola lettura.",
  },
  {
    outcome: "skipped",
    outcomeLabel: "Saltato",
    path: "Cestino / vecchio-gw",
    reason: "Elemento del cestino di Royal TSX, non incluso.",
  },
] as const;

export const report = {
  segments: [
    { outcome: "imported", label: "Importati", count: 118 },
    { outcome: "partial", label: "Parziali", count: 9 },
    { outcome: "unsupported", label: "Non supportati", count: 11 },
    { outcome: "skipped", label: "Saltati", count: 4 },
  ],
  types: [
    { label: "SSH", value: 68, kind: "ssh" },
    { label: "RDP", value: 35, kind: "rdp" },
    { label: "Cartelle", value: 16, kind: "folder" },
    { label: "Credenziali", value: 12, kind: "credential" },
    { label: "Altri tipi", value: 11, kind: "unsupported" },
  ],
} as const;

export const sessionFacts = [
  { key: "client", label: "Client", value: "FreeRDP" },
  { key: "duration", label: "Durata", value: "00:00:12", mono: true },
  { key: "host", label: "Host", value: "198.51.100.7:3389", mono: true },
];

/**
 * Imported icons. The scheme serves nothing in the gallery, so the tiles
 * show a broken image: what matters here is that they are `<img>`.
 */
export const customIcons: CustomIconOption[] = [
  { fileName: `${"a1".repeat(32)}.png`, label: "Server Royal" },
  { fileName: `${"b2".repeat(32)}.svg`, label: "Database Royal" },
];

export const pickerNodes: PickerNode[] = [
  { id: "prod", name: "Produzione", kind: "folder", depth: 0 },
  { id: "bastion", name: "bastion", kind: "ssh", depth: 1, host: "bastion.example.com" },
  { id: "web01", name: "web01", kind: "ssh", depth: 1, host: "web01.example.com", excluded: true },
  { id: "stg", name: "Staging", kind: "folder", depth: 0 },
  { id: "stg-bastion", name: "stg-bastion", kind: "ssh", depth: 1, host: "192.0.2.30" },
];

const deploy = {
  user: "deploy",
  password: "present",
  key: "Nel portachiavi, con passphrase",
} as const;

/** Every state of "Da dove arriverà" (inventario §5.6, ambiguity 23). */
export const previews: CredentialPreviewData[] = [
  {
    mode: "inherit",
    node: "web01",
    source: { chain: ["web01", "cartella Produzione", "credenziale deploy"], credential: deploy },
  },
  { mode: "inherit", node: "web01", source: null },
  { mode: "none" },
  { mode: "ask" },
  { mode: "inline", user: "admin", domain: "ACME", password: "absent" },
  {
    mode: "linked",
    credential: { name: "deploy", path: "Credenziali / deploy", resolved: deploy },
  },
  {
    mode: "byName",
    name: "deploy",
    match: { kind: "found", path: "Credenziali / deploy", resolved: deploy },
  },
  { mode: "byName", name: "deplo", match: { kind: "notFound" } },
  {
    mode: "byName",
    name: "admin",
    match: { kind: "ambiguous", paths: ["Credenziali / admin · ACME", "Staging / admin"] },
  },
];
