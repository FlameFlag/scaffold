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
          <th>Modes</th>
        </tr>
      </thead>
      <tbody>
        {#each capabilities as capability (capability.library)}
          <tr>
            <td class="capabilityLibrary">
              <code>{capability.library}</code>
              <span class="capabilityMeta">Bridge <code>{capability.bridge_library}</code></span>
              <span class="capabilityMeta">Docs <code>{capability.docs_source}</code></span>
              <p>{capability.notes}</p>
            </td>
            <td>{capability.effect}</td>
            <td class="capabilityModes">
              <span class="capabilityMode">Catalog <code>{capability.modes.catalog ?? "unavailable"}</code></span>
              <span class="capabilityMode">Test <code>{capability.modes.test ?? "unavailable"}</code></span>
              <span class="capabilityMode">Editor <code>{capability.modes.editor ?? "unavailable"}</code></span>
              <span class="capabilityMode">WASM <code>{capability.modes.wasm ?? "unavailable"}</code></span>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </figure>
</details>
