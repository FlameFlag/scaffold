import {
  EventEmitter,
  type ExtensionContext,
  MarkdownString,
  ThemeColor,
  ThemeIcon,
  type TreeDataProvider,
  TreeItem,
  TreeItemCollapsibleState,
} from "vscode";

import { scaffoldWasm } from "../wasm";
import { wasmWorkspaceJson } from "../workspace";
import { parseReferenceEntriesJson } from "./data";
import { groupedEntries, renderReferenceEntryMarkdown } from "./markdown";
import type { ReferenceEntry, ReferenceNode } from "./types";

export class ReferenceProvider implements TreeDataProvider<ReferenceNode> {
  private readonly changed = new EventEmitter<
    ReferenceNode | undefined | null
  >();
  private cachedEntries: ReferenceEntry[] | undefined;
  readonly onDidChangeTreeData = this.changed.event;

  constructor(private readonly context: ExtensionContext) {}

  refresh(): void {
    this.cachedEntries = undefined;
    this.changed.fire(undefined);
  }

  async entries(): Promise<ReferenceEntry[]> {
    if (this.cachedEntries) {
      return this.cachedEntries;
    }
    this.cachedEntries = parseReferenceEntriesJson(
      (
        await scaffoldWasm(this.context)
      ).referenceEntriesScaffoldSchemeForWorkspace(await wasmWorkspaceJson()),
    );
    return this.cachedEntries;
  }

  async searchEntries(query: string, limit: number): Promise<ReferenceEntry[]> {
    return parseReferenceEntriesJson(
      (
        await scaffoldWasm(this.context)
      ).searchReferenceEntriesScaffoldSchemeForWorkspace(
        query,
        await wasmWorkspaceJson(),
        limit,
      ),
    );
  }

  async suggestEntries(
    query: string,
    limit: number,
  ): Promise<ReferenceEntry[]> {
    return parseReferenceEntriesJson(
      (
        await scaffoldWasm(this.context)
      ).suggestReferenceEntriesScaffoldSchemeForWorkspace(
        query,
        await wasmWorkspaceJson(),
        limit,
      ),
    );
  }

  async getChildren(element?: ReferenceNode): Promise<ReferenceNode[]> {
    if (element?.kind === "group") {
      return element.entries.map((entry) => ({ kind: "entry", entry }));
    }
    return groupedEntries(await this.entries()).map((group) => ({
      kind: "group",
      name: group.name,
      entries: group.entries,
    }));
  }

  getTreeItem(element: ReferenceNode): TreeItem {
    if (element.kind === "group") {
      return groupTreeItem(element);
    }

    return entryTreeItem(element);
  }
}

function groupTreeItem(
  element: Extract<ReferenceNode, { kind: "group" }>,
): TreeItem {
  const item = new TreeItem(
    `${element.name} (${element.entries.length})`,
    TreeItemCollapsibleState.Collapsed,
  );
  item.contextValue = "scaffoldReferenceGroup";
  item.description = `${element.entries.length} entries`;
  item.tooltip = `${element.name} reference entries`;
  item.iconPath = new ThemeIcon("library");
  item.accessibilityInformation = {
    label: `${element.name}, ${element.entries.length} reference entries`,
  };
  return item;
}

function entryTreeItem(
  element: Extract<ReferenceNode, { kind: "entry" }>,
): TreeItem {
  const item = new TreeItem(element.entry.name, TreeItemCollapsibleState.None);
  item.description = entryDescription(element.entry);
  item.tooltip = referenceTooltip(element.entry);
  item.contextValue = "scaffoldReferenceEntry";
  item.iconPath = referenceIcon(element.entry);
  item.command = {
    command: "scaffold.openReferenceEntry",
    title: "Open Reference Entry",
    arguments: [element.entry],
  };
  item.accessibilityInformation = {
    label: `${element.entry.name}, ${element.entry.group} reference entry`,
  };
  return item;
}

function entryDescription(entry: ReferenceEntry): string | undefined {
  return entry.signature ?? entry.summary ?? undefined;
}

function referenceTooltip(entry: ReferenceEntry): MarkdownString {
  const markdown = new MarkdownString(renderReferenceEntryMarkdown(entry));
  markdown.supportThemeIcons = true;
  return markdown;
}

function referenceIcon(entry: ReferenceEntry): ThemeIcon {
  if (entry.deprecated) {
    return new ThemeIcon(
      "warning",
      new ThemeColor("problemsWarningIcon.foreground"),
    );
  }

  if (entry.group.toLowerCase().includes("doc")) {
    return new ThemeIcon("book");
  }

  return new ThemeIcon("symbol-function");
}
