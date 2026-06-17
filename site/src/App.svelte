<script lang="ts">
import { onMount, tick } from "svelte";
import EntryCard from "./components/EntryCard.svelte";
import InfoPanels from "./components/InfoPanels.svelte";
import ReferenceGroup from "./components/ReferenceGroup.svelte";
import Sidebar from "./components/Sidebar.svelte";
import {
  parseReferenceDocument,
  type ReferenceDocument,
  targetIdFromHash,
} from "./reference";
import { createReferenceSearchIndex } from "./reference-search";

let reference = $state<ReferenceDocument | null>(null);
let loadError = $state<string | null>(null);
let searchQuery = $state("");
let searchInput = $state<HTMLInputElement | null>(null);

async function loadReferenceDocument(): Promise<ReferenceDocument> {
  const response = await fetch(
    `${import.meta.env.BASE_URL}reference.static.json`,
  );

  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }

  return parseReferenceDocument(await response.json());
}

function isCommandShortcut(event: KeyboardEvent, key: string) {
  return (
    event.key.toLowerCase() === key &&
    (event.metaKey || event.ctrlKey) &&
    !event.altKey
  );
}

function referenceEntryCountLabel(count: number) {
  return `${count} reference entr${count === 1 ? "y" : "ies"}`;
}

function suggestedReferenceEntryCountLabel(count: number) {
  return `${count} suggested reference entr${count === 1 ? "y" : "ies"}`;
}

onMount(() => {
  let hashScrollTimer: number | undefined;

  function openSearchableContent() {
    document
      .querySelectorAll<HTMLDetailsElement>("details.searchExpandable")
      .forEach((details) => {
        details.open = true;
      });
  }

  function focusReferenceSearch() {
    searchInput?.focus();
    searchInput?.select();
  }

  function openDetailsForTarget(target: HTMLElement) {
    let current: HTMLElement | null = target;

    while (current) {
      if (current instanceof HTMLDetailsElement) {
        current.open = true;
      }
      current = current.parentElement;
    }
  }

  function targetElementById(id: string): HTMLElement | null {
    return (
      document.getElementById(id) ??
      (!id.startsWith("group-") ? document.getElementById(`group-${id}`) : null)
    );
  }

  function scrollToHash() {
    if (!window.location.hash) {
      return true;
    }

    const id = targetIdFromHash(window.location.hash);
    if (!id) {
      return true;
    }

    const target = targetElementById(id);
    if (target) {
      openDetailsForTarget(target);
    }
    target?.scrollIntoView({ block: "start" });
    return Boolean(target);
  }

  function scheduleHashScroll() {
    if (hashScrollTimer !== undefined) {
      window.clearTimeout(hashScrollTimer);
    }

    let attempts = 0;
    const retry = () => {
      attempts += 1;
      const foundTarget = scrollToHash();

      if (!foundTarget && attempts < 20) {
        hashScrollTimer = window.setTimeout(retry, 50);
        return;
      }

      if (attempts < 6) {
        hashScrollTimer = window.setTimeout(retry, 100);
      }
    };

    requestAnimationFrame(retry);
  }

  function handleFindShortcut(event: KeyboardEvent) {
    if (isCommandShortcut(event, "k")) {
      event.preventDefault();
      focusReferenceSearch();
      return;
    }

    if (isCommandShortcut(event, "f")) {
      openSearchableContent();
    }
  }

  loadReferenceDocument()
    .then(async (data) => {
      reference = data;
      await tick();
      scheduleHashScroll();
    })
    .catch((error: unknown) => {
      loadError = error instanceof Error ? error.message : String(error);
    });

  window.addEventListener("hashchange", scheduleHashScroll);
  window.addEventListener("keydown", handleFindShortcut, { capture: true });

  return () => {
    if (hashScrollTimer !== undefined) {
      window.clearTimeout(hashScrollTimer);
    }
    window.removeEventListener("hashchange", scheduleHashScroll);
    window.removeEventListener("keydown", handleFindShortcut, {
      capture: true,
    });
  };
});

let groups = $derived(
  Array.from(
    new Set(reference?.entries.map((entry) => entry.group) ?? []),
  ).sort((left, right) => left.localeCompare(right)),
);

let groupCounts = $derived(
  (reference?.entries ?? []).reduce<Record<string, number>>((counts, entry) => {
    counts[entry.group] = (counts[entry.group] ?? 0) + 1;
    return counts;
  }, Object.create(null)),
);

let searchIndex = $derived(
  reference ? createReferenceSearchIndex(reference.entries) : null,
);

let hasSearchQuery = $derived(searchQuery.trim().length > 0);
let searchResults = $derived(searchIndex?.search(searchQuery) ?? []);
let searchSuggestions = $derived(
  hasSearchQuery && searchResults.length === 0
    ? (searchIndex?.suggest(searchQuery, 5) ?? [])
    : [],
);
let searchStatus = $derived(
  reference
    ? hasSearchQuery
      ? searchResults.length > 0
        ? referenceEntryCountLabel(searchResults.length)
        : searchSuggestions.length > 0
          ? suggestedReferenceEntryCountLabel(searchSuggestions.length)
          : referenceEntryCountLabel(0)
      : referenceEntryCountLabel(reference.entries.length)
    : "",
);
</script>

<div class="app">
  <main class="content" id="content">
    <section class="landing" id="top">
      <div class="landingCopy">
        <p class="kicker">Scaffold</p>
        <h1>Scaffold Scheme Reference</h1>
        <p class="lede">
          Generated documentation for the forms Scaffold understands.
        </p>
      </div>
    </section>

    <section id="reference" class="referenceSection">
      {#if loadError}
        <p class="notice">Could not load generated reference: {loadError}</p>
      {:else if !reference}
        <p class="notice">Loading generated reference...</p>
      {:else}
        <InfoPanels capabilities={reference.capabilities} />

        <section class="referenceSearch" aria-label="Search reference">
          <label for="reference-search">Search reference</label>
          <div class="searchControl">
            <input
              id="reference-search"
              bind:this={searchInput}
              bind:value={searchQuery}
              type="search"
              autocomplete="off"
              spellcheck="false"
              placeholder="tool/path, catalog, source/path..."
            />
            {#if searchQuery.length > 0}
              <button type="button" onclick={() => (searchQuery = "")}>
                Clear
              </button>
            {/if}
          </div>
          <p aria-live="polite">
            {searchStatus}
          </p>
        </section>

        {#if hasSearchQuery}
          <section class="searchResults" aria-label="Search results">
            {#if searchResults.length > 0}
              <ol class="entryList">
                {#each searchResults as result (result.entry.name)}
                  <li>
                    <p class="searchResultMeta">{result.entry.group}</p>
                    <EntryCard entry={result.entry} />
                  </li>
                {/each}
              </ol>
            {:else}
              <p class="notice">No reference entries match this search.</p>
              {#if searchSuggestions.length > 0}
                <h2>Did you mean</h2>
                <ol class="entryList">
                  {#each searchSuggestions as result (result.entry.name)}
                    <li>
                      <p class="searchResultMeta">{result.entry.group}</p>
                      <EntryCard entry={result.entry} />
                    </li>
                  {/each}
                </ol>
              {/if}
            {/if}
          </section>
        {:else}
          {#each groups as group (group)}
            {@const entries = reference.entries.filter((entry) => entry.group === group)}
            {#if entries.length > 0}
              <ReferenceGroup {group} {entries} />
            {/if}
          {/each}
        {/if}
      {/if}
    </section>
  </main>

  <Sidebar {groups} {groupCounts} />
</div>
