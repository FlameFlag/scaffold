<script lang="ts">
import { marked } from "marked";
import { onMount, tick } from "svelte";
import InfoPanels from "./components/InfoPanels.svelte";
import ReferenceGroup from "./components/ReferenceGroup.svelte";
import Sidebar from "./components/Sidebar.svelte";
import SiteHeader from "./components/SiteHeader.svelte";
import type { ReferenceDocument } from "./reference";
import { loadReferenceDocument } from "./wasmReference";

marked.use({
  gfm: true,
  breaks: false,
});

let reference = $state<ReferenceDocument | null>(null);
let loadError = $state<string | null>(null);

onMount(() => {
  let hashScrollTimer: number | undefined;

  function scrollToHash() {
    if (!window.location.hash) {
      return true;
    }

    const id = decodeURIComponent(window.location.hash.slice(1));
    const target = document.getElementById(id);
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

  return () => {
    if (hashScrollTimer !== undefined) {
      window.clearTimeout(hashScrollTimer);
    }
    window.removeEventListener("hashchange", scheduleHashScroll);
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

let entryCount = $derived(reference?.entries.length ?? 0);
</script>

<div class="app">
  <main class="content" id="content">
    <section class="landing" id="top">
      <div class="landingCopy">
        <p class="kicker">Scaffold</p>
        <h1>Catalog your tools in a small Scheme DSL.</h1>
        <p class="lede">
          Scaffold keeps machine setup scripts readable: typed catalog helpers,
          generated reference docs, and editor intelligence from the same source.
        </p>
        <div class="landingActions">
          <a class="primaryLink" href="#reference">Browse reference</a>
          <a href="#capabilities">View capabilities</a>
        </div>
      </div>
    </section>

    <section id="reference" class="referenceSection">
    <SiteHeader
      title={reference?.title ?? "Scaffold Scheme Reference"}
      {entryCount}
      groupCount={groups.length}
      capabilityCount={reference?.capabilities.length ?? 0}
    />

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
