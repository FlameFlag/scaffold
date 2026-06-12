<script lang="ts">
import type { ReferenceCapability } from "../reference";

let { capabilities } = $props<{
  capabilities: ReferenceCapability[];
}>();
</script>

<section id="capabilities" class="panel" aria-labelledby="capabilities-title">
  <h2 id="capabilities-title">Capability Contracts</h2>
  <figure class="tableWrap">
    <table>
      <caption class="srOnly">
        Capability availability by library and execution mode.
      </caption>
      <thead>
        <tr>
          <th>Library</th>
          <th>Effect</th>
          <th>Catalog</th>
          <th>Test</th>
          <th>Editor</th>
          <th>WASM</th>
        </tr>
      </thead>
      <tbody>
        {#each capabilities as capability}
          <tr>
            <td><code>{capability.library}</code></td>
            <td>{capability.effect}</td>
            <td>{capability.modes.catalog ?? "unavailable"}</td>
            <td>{capability.modes.test ?? "unavailable"}</td>
            <td>{capability.modes.editor ?? "unavailable"}</td>
            <td>{capability.modes.wasm ?? "unavailable"}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </figure>
</section>

<section id="catalog-schema" class="panel" aria-labelledby="catalog-schema-title">
  <h2 id="catalog-schema-title">Catalog Schema</h2>
  <p>
    The generated reference includes schema metadata used by Scaffold catalog
    validation.
  </p>
</section>
