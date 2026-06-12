import type { DocumentSelector, TextDocument } from "vscode";

export const schemeSelector: DocumentSelector = [
  { language: "scaffold-scheme" },
  { pattern: "**/*.scm" },
];

export function isSchemeDocument(document: TextDocument): boolean {
  return (
    document.languageId === "scaffold-scheme" ||
    document.uri.path.endsWith(".scm")
  );
}
