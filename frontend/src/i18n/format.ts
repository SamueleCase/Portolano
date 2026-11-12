/** The two forms of a text with a count: "1 nodo", "{n} nodi". */
export interface PluralForms {
  one: string;
  other: string;
}

const numbers = new Intl.NumberFormat("it-IT");

/**
 * `template` with each `{name}` replaced by its value. Values are inserted as
 * they are: the components show the result as text, never as markup.
 */
export function interpolate(template: string, values: Readonly<Record<string, string | number>>) {
  return template.replaceAll(/\{(\w+)\}/g, (placeholder, name: string) => {
    const value = values[name];
    if (value === undefined) {
      return placeholder;
    }
    return typeof value === "number" ? numbers.format(value) : value;
  });
}

/** The form of `forms` for `count`, with `{n}` replaced by the count (SPEC-UI §5.1). */
export function plural(count: number, forms: PluralForms): string {
  return interpolate(count === 1 ? forms.one : forms.other, { n: count });
}
