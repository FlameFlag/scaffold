import { containsUnsafeRenderedHtml } from "../src/rendered-html.ts";

export function assertSafeRenderedHtmlEntries(entries) {
  const unsafeHtml = renderedHtmlPayloads(entries).find((payload) =>
    containsUnsafeRenderedHtml(payload.html),
  );
  if (unsafeHtml) {
    throw new Error(`Unsafe rendered HTML in ${unsafeHtml.entry}`);
  }
}

function renderedHtmlPayloads(entries) {
  return entries
    .flatMap((entry) => [
      {
        entry: `${entry.name}.markdown`,
        html: entry.rendered?.rawMarkdownHtml,
      },
      { entry: `${entry.name}.returns`, html: entry.rendered?.returnsHtml },
      {
        entry: `${entry.name}.source`,
        html: entry.rendered?.sourceSnippet?.html,
      },
      ...(entry.rendered?.params ?? []).map((param) => ({
        entry: `${entry.name}.param.${param.name}`,
        html: param.summaryHtml,
      })),
    ])
    .filter((payload) => typeof payload.html === "string");
}
