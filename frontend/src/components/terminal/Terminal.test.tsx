import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { Terminal } from "./Terminal";
import type { TerminalIo } from "./terminalIo";

// xterm.js needs a real layout engine: here a stand-in records how the
// component wires it, and the gallery e2e runs the real one in WebKit.
interface FakeXterm {
  options: unknown;
  osc: Map<number, (data: string) => boolean>;
  title: ((title: string) => void) | undefined;
  data: ((data: string) => void) | undefined;
  written: Uint8Array[];
  disposed: boolean;
  keys: ((event: KeyboardEvent) => boolean) | undefined;
}

const fake = vi.hoisted((): FakeXterm => ({
  options: undefined,
  osc: new Map(),
  title: undefined,
  data: undefined,
  written: [],
  disposed: false,
  keys: undefined,
}));

vi.mock("@xterm/xterm", () => ({
  Terminal: class {
    parser = {
      registerOscHandler: (ident: number, handler: (data: string) => boolean) => {
        fake.osc.set(ident, handler);
        return { dispose: () => undefined };
      },
    };
    constructor(options: unknown) {
      fake.options = options;
    }
    textarea = undefined;
    loadAddon() {}
    open() {}
    attachCustomKeyEventHandler(handler: (event: KeyboardEvent) => boolean) {
      fake.keys = handler;
    }
    write(data: Uint8Array) {
      fake.written.push(data);
    }
    onTitleChange(listener: (title: string) => void) {
      fake.title = listener;
      return { dispose: () => undefined };
    }
    onData(listener: (data: string) => void) {
      fake.data = listener;
      return { dispose: () => undefined };
    }
    onResize() {
      return { dispose: () => undefined };
    }
    dispose() {
      fake.disposed = true;
    }
  },
}));
vi.mock("@xterm/addon-fit", () => ({
  FitAddon: class {
    fit() {}
  },
}));
vi.mock("@xterm/addon-web-links", () => ({ WebLinksAddon: class {} }));

function fakeIo() {
  let output: ((data: Uint8Array) => void) | undefined;
  const input = vi.fn<(data: string) => void>();
  const io: TerminalIo = {
    subscribe: (onOutput) => {
      output = onOutput;
      return () => {
        output = undefined;
      };
    },
    input,
    resize: vi.fn(),
  };
  return { io, input, emit: (data: Uint8Array) => output?.(data) };
}

interface LinkOptions {
  linkHandler: { activate: (event: MouseEvent, uri: string) => void };
}

describe("Terminal", () => {
  beforeEach(() => {
    fake.osc.clear();
    fake.written = [];
    fake.disposed = false;
  });

  it("is a group named Terminale", () => {
    render(<Terminal io={fakeIo().io} onOpenLink={vi.fn()} />);

    expect(screen.getByRole("group", { name: "Terminale" })).toBeInTheDocument();
  });

  it("writes the output of the session", () => {
    const { io, emit } = fakeIo();
    render(<Terminal io={io} onOpenLink={vi.fn()} />);

    emit(new TextEncoder().encode("uptime\r\n"));

    expect(fake.written).toHaveLength(1);
  });

  it("sends what the user types and reports the activity", () => {
    const { io, input } = fakeIo();
    const onActivity = vi.fn();
    render(<Terminal io={io} onOpenLink={vi.fn()} onActivity={onActivity} />);

    fake.data?.("ls\r");

    expect(input).toHaveBeenCalledWith("ls\r");
    expect(onActivity).toHaveBeenCalledOnce();
  });

  it("swallows OSC 52, so the host cannot touch the clipboard", () => {
    render(<Terminal io={fakeIo().io} onOpenLink={vi.fn()} />);

    expect(fake.osc.get(52)?.("c;aGVsbG8=")).toBe(true);
  });

  it("passes the title on as visible text", () => {
    const onTitle = vi.fn();
    render(<Terminal io={fakeIo().io} onOpenLink={vi.fn()} onTitle={onTitle} />);

    fake.title?.("db02\u0007");

    expect(onTitle).toHaveBeenCalledWith("db02\\u{7}");
  });

  it("opens only http(s) links", () => {
    const onOpenLink = vi.fn();
    render(<Terminal io={fakeIo().io} onOpenLink={onOpenLink} />);
    const { linkHandler } = fake.options as LinkOptions;

    linkHandler.activate(new MouseEvent("click"), "file:///etc/hosts");
    linkHandler.activate(new MouseEvent("click"), "https://www.example.com/");

    expect(onOpenLink).toHaveBeenCalledExactlyOnceWith("https://www.example.com/");
  });

  it("leaves the terminal with Ctrl+Tab, as Tab belongs to the shell", () => {
    render(
      <>
        <button type="button">prima</button>
        <Terminal io={fakeIo().io} onOpenLink={vi.fn()} />
        <button type="button">dopo</button>
      </>,
    );

    const handled = fake.keys?.(new KeyboardEvent("keydown", { key: "Tab", ctrlKey: true }));

    expect(handled).toBe(false);
    expect(screen.getByRole("button", { name: "dopo" })).toHaveFocus();
  });

  it("leaves a plain Tab to the shell", () => {
    render(<Terminal io={fakeIo().io} onOpenLink={vi.fn()} />);

    expect(fake.keys?.(new KeyboardEvent("keydown", { key: "Tab" }))).toBe(true);
  });

  it("releases xterm when unmounted", () => {
    const { unmount } = render(<Terminal io={fakeIo().io} onOpenLink={vi.fn()} />);

    unmount();

    expect(fake.disposed).toBe(true);
  });
});
