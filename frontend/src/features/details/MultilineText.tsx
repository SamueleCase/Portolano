import { Fragment } from "react";

import { visibleText } from "../../components/visibleText";

/**
 * A text of several lines from the document (notes, the original of an
 * unsupported node), line by line: line ends and tabs keep their meaning,
 * every other control is made visible as by `visibleText` (SPEC-UI §1.3).
 */
export function MultilineText({ text }: { text: string }) {
  const lines = text.split(/\r?\n/);
  return lines.map((line, index) => (
    <Fragment key={index}>
      {line.split("\t").map(visibleText).join("\t")}
      {index < lines.length - 1 && "\n"}
    </Fragment>
  ));
}
