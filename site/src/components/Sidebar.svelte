<script lang="ts">
import { anchor } from "../reference";

let { groups, groupCounts } = $props<{
  groups: string[];
  groupCounts: Record<string, number>;
}>();

type SidebarSection = {
  type: "section";
  name: string;
  groups: string[];
};

type SidebarItem = SidebarSection;

const sectionPlan = [
  {
    name: "Core",
    groups: ["Language", "Objects", "Vectors", "Paths", "Filesystem", "Host", "Workspace"],
  },
  {
    name: "Catalog",
    groups: ["Catalog", "Actions", "Documentation", "Checks", "Testing", "Transformations"],
  },
  {
    name: "Packages",
    groups: [
      "Applications",
      "Archives",
      "Build tools",
      "Distro packages",
      "Bun",
      "Cargo",
      "npm",
      "uv",
      "Platforms",
      "Targets",
    ],
  },
  {
    name: "Nix",
    groups: ["Nix", "Nix build", "Nix eval", "Nix flakes", "Nix profiles", "Nix shell", "Nix store"],
  },
  {
    name: "OS tools",
    groups: ["macOS tools", "Windows tools"],
  },
];

let groupedNav = $derived(buildGroupedNav(groups));

function buildGroupedNav(values: string[]) {
  const items: SidebarItem[] = [];
  const used = new Set<string>();

  for (const section of sectionPlan) {
    const sectionGroups = section.groups.filter((group) => values.includes(group));

    if (sectionGroups.length > 0) {
      items.push({ type: "section", name: section.name, groups: sectionGroups });
      sectionGroups.forEach((group) => used.add(group));
    }
  }

  const unplanned = values.filter((group) => !used.has(group));
  if (unplanned.length > 0) {
    items.push({ type: "section", name: "Other", groups: unplanned });
  }

  return items;
}

function sectionCount(section: SidebarSection) {
  return section.groups.reduce((count, group) => count + (groupCounts[group] ?? 0), 0);
}

function childLabel(section: SidebarSection, group: string) {
  return group === section.name ? "Overview" : group.slice(section.name.length + 1);
}

function labelFor(section: SidebarSection, group: string) {
  if (section.name === "Nix") {
    return childLabel(section, group);
  }

  return group;
}
</script>

<aside class="sidebar" aria-label="Reference index">
  <a class="skipLink" href="#content">Skip to reference</a>
  <a class="brand" href="#top">Scaffold</a>
  <nav aria-label="Reference sections">
    <ul class="navList">
      <li><a href="#reference">Reference</a></li>
      <li><a href="#capabilities">Capabilities</a></li>
    {#each groupedNav as item}
      <li>
        <details class="navGroup">
          <summary>
            <span>{item.name}</span>
            <small>{sectionCount(item)}</small>
          </summary>
          <ul>
            {#each item.groups as group}
              <li>
                <a href={`#${anchor(group)}`}>
                  <span>{labelFor(item, group)}</span>
                  <small>{groupCounts[group]}</small>
                </a>
              </li>
            {/each}
          </ul>
        </details>
      </li>
    {/each}
    </ul>
  </nav>
</aside>
