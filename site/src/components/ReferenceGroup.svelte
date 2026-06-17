<script lang="ts">
import { groupId, type ReferenceEntry } from "../reference";
import EntryCard from "./EntryCard.svelte";

let { group, entries } = $props<{
  group: string;
  entries: ReferenceEntry[];
}>();
</script>

<details
  class="referenceGroup searchExpandable"
  id={groupId(group)}
  aria-labelledby={`${groupId(group)}-title`}
>
  <summary class="groupSummary">
    <h2 id={`${groupId(group)}-title`}>{group}</h2>
    <p>{entries.length} entries</p>
  </summary>
  <ol class="entryList" aria-label={`${group} reference entries`}>
    {#each entries as entry (entry.name)}
      <li>
        <EntryCard {entry} />
      </li>
    {/each}
  </ol>
</details>
