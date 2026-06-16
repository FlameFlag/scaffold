<script lang="ts">
import type { ReferenceEntry } from "../reference";
import CopyButton from "./CopyButton.svelte";

let { entry } = $props<{ entry: ReferenceEntry }>();

const snippet = $derived(entry.rendered.sourceSnippet);
</script>

{#if snippet}
  <details class="sourceDisclosure searchExpandable">
    <summary>
      <span>Source</span>
      <small>{snippet.label}</small>
    </summary>

    <div class="sourceToolbar">
      <CopyButton value={snippet.code} label="Copy source" />
    </div>

    <div class="sourceCode" style={`--source-start: ${snippet.startLine - 1}`}>
      {@html snippet.html}
    </div>
  </details>
{/if}
