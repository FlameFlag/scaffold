<script lang="ts">
import { anchor, type ReferenceEntry } from "../reference";
import EntryCard from "./EntryCard.svelte";

let { group, entries } = $props<{
  group: string;
  entries: ReferenceEntry[];
}>();
</script>

<section class="referenceGroup" id={anchor(group)} aria-labelledby={`${anchor(group)}-title`}>
  <header>
    <h2 id={`${anchor(group)}-title`}>{group}</h2>
    <p>{entries.length} entries</p>
  </header>
  <ol class="entryList" aria-label={`${group} reference entries`}>
    {#each entries as entry}
      <li>
        <EntryCard {entry} />
      </li>
    {/each}
  </ol>
</section>
