<script lang="ts">
import { marked } from "marked";
import { anchor, type ReferenceEntry } from "../reference";
import SourceDisclosure from "./SourceDisclosure.svelte";

let { entry } = $props<{ entry: ReferenceEntry }>();

function markdownHtml(value: string) {
  return String(marked.parse(value));
}

function inlineMarkdownHtml(value: string) {
  return String(marked.parseInline(value));
}
</script>

<article class="entry" id={`entry-${anchor(entry.name)}`}>
  <header>
    <h3><code>{entry.name}</code></h3>
    <span>{entry.kind}</span>
  </header>

  {#if entry.signature}
    <pre>{entry.signature}</pre>
  {/if}

  {#if entry.summary}
    <p class="summary">{entry.summary}</p>
  {/if}

  {#if entry.markdown}
    <section class="markdown" aria-label="Documentation">
      {@html markdownHtml(entry.markdown)}
    </section>
  {/if}

  {#if entry.params.length > 0 || entry.returns}
    <dl class="params">
      {#each entry.params as param}
        <dt>{param.name}</dt>
        <dd>{@html inlineMarkdownHtml(param.summary)}</dd>
      {/each}
      {#if entry.returns}
        <dt>Returns</dt>
        <dd>{@html inlineMarkdownHtml(entry.returns)}</dd>
      {/if}
    </dl>
  {/if}

  {#if entry.example}
    <pre class="example"><code>{entry.example}</code></pre>
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
</article>
