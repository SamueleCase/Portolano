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

const percents = new Intl.NumberFormat("it-IT", {
  style: "percent",
  minimumFractionDigits: 1,
  maximumFractionDigits: 1,
});

/** A share as an Italian percentage with one decimal: "83,1%" (inventario §6). */
export function percent(part: number, total: number): string {
  return percents.format(total === 0 ? 0 : part / total);
}

/** The form of `forms` for `count`, with `{n}` replaced by the count (SPEC-UI §5.1). */
export function plural(count: number, forms: PluralForms): string {
  return interpolate(count === 1 ? forms.one : forms.other, { n: count });
}
