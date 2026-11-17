import { useRef } from "react";

import { Button } from "../../components/Button";
import { Dialog } from "../../components/Dialog";
import { TextField } from "../../components/TextField";
import type { LoginInput_Deserialize, LoginRequest } from "../../bindings";
import { texts } from "../../i18n/it";
import styles from "./Dialogs.module.css";
import { readablePath } from "../tree/treeModel";

interface LoginDialogProps {
  /** What `login_request` asked; `null` closes the dialog. */
  request: LoginRequest | null;
  /**
   * The login typed, handed over once: the dialog empties its fields right
   * after the call and keeps nothing.
   */
  onLogin: (login: LoginInput_Deserialize) => void;
  onClose: () => void;
}

/** An empty field is no value: the client asks for it, as the CLI leaves it. */
function typed(field: HTMLInputElement | null): string | null {
  const value = field?.value ?? "";
  return value === "" ? null : value;
}

/**
 * The credentials of a connection whose credential is "Chiedi all'avvio"
 * (DEC-016.3, inventario §5.17): user, domain when the protocol uses one,
 * password. The fields are uncontrolled and never become React state.
 */
export function LoginDialog({ request, onLogin, onClose }: LoginDialogProps) {
  // A new dialog for each request: nothing typed for one connection stays
  // in the fields of the next.
  return request === null ? null : (
    <OpenLoginDialog key={request.node} request={request} onLogin={onLogin} onClose={onClose} />
  );
}

function OpenLoginDialog({
  request,
  onLogin,
  onClose,
}: LoginDialogProps & { request: LoginRequest }) {
  const t = texts.dialogs.login;
  const user = useRef<HTMLInputElement>(null);
  const domain = useRef<HTMLInputElement>(null);
  const password = useRef<HTMLInputElement>(null);

  const submit = () => {
    const login = {
      username: typed(user.current),
      domain: typed(domain.current),
      password: typed(password.current),
    };
    for (const field of [user, domain, password]) {
      if (field.current) field.current.value = "";
    }
    onLogin(login);
  };

  return (
    <Dialog
      open
      onClose={onClose}
      title={t.title}
      subtitle={readablePath(request.path)}
      width="small"
      onSubmit={submit}
      footer={
        <>
          <Button onClick={onClose}>{texts.dialogs.cancel}</Button>
          <Button variant="primary" type="submit">
            {t.connect}
          </Button>
        </>
      }
    >
      <p className={styles.hint}>{t.text}</p>
      {request.ask_domain ? (
        <div className={styles.pair}>
          <TextField ref={user} label={t.user} mono autoComplete="off" />
          <TextField ref={domain} label={t.domain} mono autoComplete="off" />
        </div>
      ) : (
        <TextField ref={user} label={t.user} mono autoComplete="off" />
      )}
      <TextField ref={password} label={t.password} type="password" autoComplete="off" />
    </Dialog>
  );
}
