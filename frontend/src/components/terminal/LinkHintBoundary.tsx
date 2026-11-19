import { Component, type ReactNode } from "react";

interface LinkHintBoundaryProps {
  children: ReactNode;
}

interface LinkHintBoundaryState {
  failed: boolean;
}

/**
 * Keeps an error of the hint of a terminal link inside the hint: the hint
 * goes, the terminal and the rest of the window stay (DEC-047.6). The text
 * of the link comes from the remote host, so the hint must never take the
 * window down with it. The terminal mounts a new boundary for each link.
 */
export class LinkHintBoundary extends Component<LinkHintBoundaryProps, LinkHintBoundaryState> {
  override state: LinkHintBoundaryState = { failed: false };

  static getDerivedStateFromError(): LinkHintBoundaryState {
    return { failed: true };
  }

  override render(): ReactNode {
    return this.state.failed ? null : this.props.children;
  }
}
