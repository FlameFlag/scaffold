<script lang="ts">
import { anchor, type ReferenceEntry } from "../reference";
import EntryCard from "./EntryCard.svelte";

let { group, entries } = $props<{
  group: string;
  entries: ReferenceEntry[];
}>();
</script>

<details
  class="referenceGroup searchExpandable"
  id={anchor(group)}
  aria-labelledby={`${anchor(group)}-title`}
>
  <summary class="groupSummary">
    <h2 id={`${anchor(group)}-title`}>{group}</h2>
    <p>{entries.length} entries</p>
  </summary>
  <ol class="entryList" aria-label={`${group} reference entries`}>
    {#each entries as entry}
      <li>
        <EntryCard {entry} />
      </li>
    {/each}
  </ol>
</details>
