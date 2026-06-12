<script lang="ts">
import type { ReferenceEntry } from "../reference";
import { highlightScaffoldScheme, sourceSnippet } from "../wasmReference";

let { entry } = $props<{ entry: ReferenceEntry }>();

let highlighted = $state<string | null>(null);
let highlightError = $state<string | null>(null);

const snippet = $derived(sourceSnippet(entry));

type SchemeHighlighter = {
  codeToHtml(code: string): Promise<string>;
};

let highlighter: Promise<SchemeHighlighter> | undefined;

function schemeHighlighter() {
  highlighter ??= Promise.resolve({
    codeToHtml: highlightScaffoldScheme,
  });
  return highlighter;
}

async function highlightSource() {
  if (!snippet || highlighted || highlightError) {
    return;
  }

  try {
    highlighted = await (await schemeHighlighter()).codeToHtml(snippet.code);
  } catch (error) {
    highlightError = error instanceof Error ? error.message : String(error);
  }
}
</script>

{#if snippet}
  <details
    class="sourceDisclosure"
    ontoggle={(event) => {
      if ((event.currentTarget as HTMLDetailsElement).open) {
        void highlightSource();
      }
    }}
  >
    <summary>
      <span>Source</span>
      <small>{snippet.label}</small>
    </summary>

    {#if highlighted}
      <div
        class="sourceCode"
        style={`--source-start: ${snippet.startLine - 1}`}
      >
        {@html highlighted}
      </div>
    {:else if highlightError}
      <pre class="sourceFallback"><code>{snippet.code}</code></pre>
    {:else}
      <p class="sourceLoading">Loading source...</p>
    {/if}
  </details>
{/if}
