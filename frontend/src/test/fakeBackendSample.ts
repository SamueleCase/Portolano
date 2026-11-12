import {
  type FakeBackendState,
  type FakeDocument,
  type FakeNode,
  blankNode,
} from "./fakeBackendModel";

/**
 * The sample document of the fake backend: folders, SSH and RDP connections,
 * credentials, a jump host and an unsupported node. Invented data only
 * (SPEC §1.3): hosts under `example.com` and RFC 5737, fake users.
 */
export function sampleDocument(state: FakeBackendState, path: string): FakeDocument {
  const node = (kind: FakeNode["kind"], name: string, fields: Partial<FakeNode> = {}): FakeNode =>
    Object.assign(blankNode(state, kind, name), fields);

  const admin = node("credential", "Admin ACME", {
    username: "administrator",
    domain: "ACME",
    secrets: { password: true },
  });
  const deploy = node("credential", "deploy", {
    username: "deploy",
    keyFile: "~/.ssh/id_ed25519",
    secrets: { passphrase: false },
  });
  const bastion = node("connection", "bastion", {
    protocol: "ssh",
    host: "bastion.example.com",
    credential: { mode: "byId", id: deploy.id },
    tags: ["rete"],
  });
  const web01 = node("connection", "web01", {
    protocol: "ssh",
    host: "web01.example.com",
    port: 2222,
    favorite: true,
    tags: ["web", "produzione"],
    color: "#4F9DDE",
    icon: { kind: "builtin", name: "server" },
    settings: { jump_hosts: [{ $ref: bastion.id }], agent_forwarding: true },
    description: "Server web principale",
    notes: "Riavvio solo nella finestra di manutenzione.",
  });
  const dc01 = node("connection", "dc01", {
    protocol: "rdp",
    host: "192.0.2.10",
    icon: { kind: "royal", name: "Flat/Hardware/Server" },
  });
  const kiosk = node("connection", "kiosk", {
    protocol: "rdp",
    host: "198.51.100.30",
    credential: { mode: "prompt" },
  });
  const acme = node("folder", "ACME", {
    credential: { mode: "byId", id: admin.id },
    children: [web01, dc01, kiosk],
  });
  const clienti = node("folder", "Clienti", { children: [acme, admin] });
  const rete = node("folder", "Rete", { children: [bastion, deploy] });
  const vnc = node("unsupported", "vnc-legacy", {
    unsupported: {
      sourceType: "RoyalVNCConnection",
      raw: '<RoyalVNCConnection Name="vnc-legacy" />',
    },
    source: {
      system: "royal",
      sourceType: "RoyalVNCConnection",
      sourceId: "royal-vnc",
      file: "infra.rtsz",
    },
  });
  return { name: "Infrastruttura", path, nodes: [clienti, rete, vnc] };
}
