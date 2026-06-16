<script lang="ts">
import { anchor, type ReferenceEntry } from "../reference";
import CopyButton from "./CopyButton.svelte";
import SourceDisclosure from "./SourceDisclosure.svelte";

let { entry } = $props<{ entry: ReferenceEntry }>();

function renderedParamSummary(name: string) {
  return (
    entry.rendered.params.find((item) => item.name === name)?.summaryHtml ?? ""
  );
}
</script>

<details class="entry searchExpandable" id={`entry-${anchor(entry.name)}`}>
  <summary class="entrySummary">
    <span class="entryTitle">
      <h3><code>{entry.name}</code></h3>
      <span>{entry.kind}</span>
    </span>
    {#if entry.summary}
      <span class="summary">{entry.summary}</span>
    {/if}
  </summary>

  <div class="entryBody">
    {#if entry.signature}
      <div class="copyBlock">
        <div class="copyToolbar">
          <CopyButton value={entry.signature} />
        </div>
        <pre>{entry.signature}</pre>
      </div>
    {/if}

    {#if entry.markdown}
      <section class="markdown" aria-label="Documentation">
        {@html entry.rendered.markdownHtml}
      </section>
    {/if}

    {#if entry.params.length > 0 || entry.returns}
      <dl class="params">
        {#each entry.params as param}
          <dt>{param.name}</dt>
          <dd>{@html renderedParamSummary(param.name)}</dd>
        {/each}
        {#if entry.returns}
          <dt>Returns</dt>
          <dd>{@html entry.rendered.returnsHtml ?? ""}</dd>
        {/if}
      </dl>
    {/if}

    {#if entry.example}
      <div class="copyBlock">
        <div class="copyToolbar">
          <CopyButton value={entry.example} />
        </div>
        <pre class="example"><code>{entry.example}</code></pre>
      </div>
    {/if}

    <footer>
      {#if entry.effect}
        <span>{entry.effect}</span>
      {/if}
      {#if entry.stability}
        <span>{entry.stability}</span>
      {/if}
      {#if entry.since}
        <span>since {entry.since}</span>
      {/if}
    </footer>

    <SourceDisclosure {entry} />
  </div>
</details>
