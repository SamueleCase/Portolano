import { queryOptions } from "@tanstack/react-query";

import { commands } from "../bindings";
import { unwrap } from "./result";

/** The default document and the vault backend of the configuration. */
export const appStatusQuery = queryOptions({
  queryKey: ["app-status"],
  queryFn: async () => unwrap(await commands.appStatus()),
});
