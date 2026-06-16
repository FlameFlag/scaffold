<script lang="ts">
import { onMount, tick } from "svelte";
import InfoPanels from "./components/InfoPanels.svelte";
import ReferenceGroup from "./components/ReferenceGroup.svelte";
import Sidebar from "./components/Sidebar.svelte";
import type { ReferenceDocument } from "./reference";

let reference = $state<ReferenceDocument | null>(null);
let loadError = $state<string | null>(null);

async function loadReferenceDocument(): Promise<ReferenceDocument> {
  const response = await fetch(`${import.meta.env.BASE_URL}reference.static.json`);

  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }

  return (await response.json()) as ReferenceDocument;
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

  function openDetailsForTarget(target: HTMLElement) {
    let current: HTMLElement | null = target;

    while (current) {
      if (current instanceof HTMLDetailsElement) {
        current.open = true;
      }
      current = current.parentElement;
    }
  }

  function scrollToHash() {
    if (!window.location.hash) {
      return true;
    }

    const id = decodeURIComponent(window.location.hash.slice(1));
    const target = document.getElementById(id);
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
    if (
      event.key.toLowerCase() === "f" &&
      (event.metaKey || event.ctrlKey) &&
      !event.altKey
    ) {
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
    window.removeEventListener("keydown", handleFindShortcut, { capture: true });
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
  }, {}),
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

        {#each groups as group}
          {@const entries = reference.entries.filter((entry) => entry.group === group)}
          {#if entries.length > 0}
            <ReferenceGroup {group} {entries} />
          {/if}
        {/each}
      {/if}
    </section>
  </main>

  <Sidebar {groups} {groupCounts} />
</div>
