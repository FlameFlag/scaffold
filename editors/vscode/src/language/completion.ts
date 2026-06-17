import {
  CompletionItem,
  CompletionItemKind,
  CompletionItemTag,
  type Disposable,
  type ExtensionContext,
  languages,
  MarkdownString,
  SnippetString,
} from "vscode";

import { schemeSelector } from "../scheme";
import { scaffoldWasm } from "../wasm";
import { parseWasmJson } from "../wasm/json";
import { wasmWorkspaceJson } from "../workspace";
import {
  isWasmCompletionItem,
  type WasmCompletionItem,
} from "./completion-data";

export function registerCompletionProvider(
  context: ExtensionContext,
): Disposable {
  return languages.registerCompletionItemProvider(schemeSelector, {
    async provideCompletionItems(document) {
      return parseWasmJson<WasmCompletionItem[]>(
        (await scaffoldWasm(context)).completionItemsScaffoldSchemeForDocument(
          document.getText(),
          await wasmWorkspaceJson(),
        ),
        [],
      )
        .filter(isWasmCompletionItem)
        .map(toVsCodeCompletionItem);
    },
  });
}

function toVsCodeCompletionItem(item: WasmCompletionItem): CompletionItem {
  const completion = new CompletionItem(
    item.label,
    item.kind === "keyword"
      ? CompletionItemKind.Keyword
      : CompletionItemKind.Function,
  );
  completion.detail = item.detail;
  completion.documentation = new MarkdownString(item.documentation);
  completion.insertText = item.insert_text_is_snippet
    ? new SnippetString(item.insert_text)
    : item.insert_text;
  completion.sortText = item.sort_text;
  if (item.deprecated) {
    completion.tags = [CompletionItemTag.Deprecated];
  }
  return completion;
}
