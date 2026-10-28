<script lang="ts">
  import { ROLES, type Contributor, type EpisodePerson } from "$lib/site";

  interface Props {
    /** Who and in which role; someone in several roles has a row per role. */
    people: EpisodePerson[];
    contributors: Contributor[];
    /** For ids, so several editors can be on one page. */
    id: string;
  }

  let { people = $bindable(), contributors, id }: Props = $props();

  function add() {
    // The first contributor not in the list yet, as a guest; else the first.
    const fresh = contributors.find((c) => !people.some((p) => p.contributor === c.id)) ?? contributors[0];
    if (fresh) people = [...people, { contributor: fresh.id, role: people.length === 0 ? "host" : "guest" }];
  }

  function twice(index: number): boolean {
    const row = people[index];
    return people.slice(0, index).some((p) => p.contributor === row.contributor && p.role === row.role);
  }
</script>

<div id={id} class="flex flex-col gap-2">
  {#each people as person, index (index)}
    <div class="grid grid-cols-[1fr_12rem_auto] gap-2">
      <select class="select w-full" bind:value={person.contributor} aria-label="Contributor">
        {#each contributors as contributor (contributor.id)}
          <option value={contributor.id}>{contributor.name}</option>
        {/each}
        {#if !contributors.some((c) => c.id === person.contributor)}
          <option value={person.contributor}>{person.contributor} (missing)</option>
        {/if}
      </select>
      <select class={["select w-full", twice(index) && "select-warning"]} bind:value={person.role} aria-label="Role"
        title={twice(index) ? "Listed twice in this role" : undefined}>
        {#each ROLES as role (role.value)}<option value={role.value}>{role.label}</option>{/each}
      </select>
      <button type="button" class="btn btn-ghost" aria-label="Remove"
        onclick={() => (people = people.filter((_, i) => i !== index))}>Remove</button>
    </div>
  {/each}
  {#if contributors.length}
    <div>
      <button type="button" id="{id}-add" class="btn btn-sm" onclick={add}>Add a person</button>
      <span class="ms-2 text-sm text-base-content/60">Someone in several roles gets a row for each.</span>
    </div>
  {:else}
    <p class="text-sm text-base-content/60">No contributors yet: add them in the Contributors tab first.</p>
  {/if}
</div>
