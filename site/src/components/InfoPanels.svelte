<script lang="ts">
import type { ReferenceCapability } from "../reference";

let { capabilities } = $props<{
  capabilities: ReferenceCapability[];
}>();
</script>

<details id="capabilities" class="panel panelDisclosure searchExpandable">
  <summary>
    <span>
      <span class="summaryKicker">Runtime support</span>
      <h2 id="capabilities-title">Capability contracts</h2>
    </span>
    <small>{capabilities.length} libraries</small>
  </summary>
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
</details>
