import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { texts } from "../../i18n/it";
import { SessionView, type SessionViewProps } from "./SessionView";
import type { Session } from "./store";

const session: Session = {
  id: "session-1",
  nodeId: "node-rdp",
  kind: "rdp-external",
  state: { state: "connected" },
  startedAt: 0,
};

function RdpStub({ session }: SessionViewProps) {
  return <p>vista RDP di {session.nodeId}</p>;
}

describe("SessionView", () => {
  it("shows the view registered for the kind of the session", () => {
    render(<SessionView session={session} summary="dc01" views={{ "rdp-external": RdpStub }} />);

    expect(screen.getByText("vista RDP di node-rdp")).toBeInTheDocument();
  });

  it("says when a kind has no view yet", () => {
    render(
      <SessionView
        session={{ ...session, kind: "rdp-embedded" }}
        summary="dc01"
        views={{ "rdp-external": RdpStub }}
      />,
    );

    expect(screen.getByRole("status")).toHaveTextContent(texts.sessions.missingView);
  });
});
