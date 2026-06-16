<script lang="ts">
let { title, entryCount, groupCount, capabilityCount } = $props<{
  title: string;
  entryCount: number;
  groupCount: number;
  capabilityCount: number;
}>();
</script>

<header class="siteHeader">
  <p class="kicker">Scaffold docs</p>
  <h2>{title}</h2>
  <p class="lede">
    A compact reference for the Scheme forms Scaffold understands.
  </p>
  <dl class="stats" aria-label="Reference stats">
    <dt>Entries</dt>
    <dd>{entryCount}</dd>
    <dt>Groups</dt>
    <dd>{groupCount}</dd>
    <dt>Capabilities</dt>
    <dd>{capabilityCount}</dd>
  </dl>
</header>
