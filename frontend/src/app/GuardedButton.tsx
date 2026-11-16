import type { ComponentProps } from "react";

import { Button } from "../components/Button";
import { Tooltip } from "../components/Tooltip";

interface GuardedButtonProps extends ComponentProps<typeof Button> {
  /** Why the button is disabled; the button is offered only without one. */
  reason?: string | undefined;
  /** The text of the tooltip of an enabled button, with `shortcut`. */
  hint?: string | undefined;
  /** The shortcut of the menu, such as "⌘↩", in the tooltip. */
  shortcut?: string | undefined;
}

/**
 * A button of a command that may not be offered: disabled with the reason
 * in its tooltip (DEC-035: no placeholder), otherwise with its shortcut.
 */
export function GuardedButton({ reason, hint, shortcut, disabled, ...rest }: GuardedButtonProps) {
  const button = <Button {...rest} disabled={disabled === true || reason !== undefined} />;
  if (reason !== undefined) {
    return <Tooltip content={reason}>{button}</Tooltip>;
  }
  if (hint !== undefined) {
    return (
      <Tooltip content={hint} shortcut={shortcut}>
        {button}
      </Tooltip>
    );
  }
  return button;
}
