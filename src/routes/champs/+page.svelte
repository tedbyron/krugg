<script lang="ts">
  import { api, type ChampionShort } from '$lib'
  import X from '~icons/tabler/x'

  let search = $state.raw('')
  let filteredChamps = $state.raw<ChampionShort[]>()

  $effect(() => {
    // TODO: levenshtein
    if (api.champs !== undefined && search !== '') {
      filteredChamps = api.champs.filter(({ name }) =>
        name.toLowerCase().includes(search.toLowerCase()),
      )
    } else {
      filteredChamps = api.champs
    }
  })
</script>

{#if filteredChamps !== undefined}
  <div class="container flex flex-col gap-2">
    <div class="flex justify-center">
      <div class="relative w-1/2 max-w-md">
        <!-- Search input -->
        <input
          type="text"
          autocapitalize="words"
          placeholder="Search"
          bind:value={search}
          class="w-full rounded-lg border bg-gruvbox-bg py-1 pl-2 pr-6 text-center leading-none text-gruvbox-fg placeholder:text-center placeholder:text-gruvbox-gray dark:bg-gruvbox-dark-bg dark:text-gruvbox-dark-fg dark:placeholder:text-gruvbox-dark-gray"
        />

        <!-- Clear input button -->
        <button
          type="button"
          onclick={() => {
            search = ''
          }}
          class={['absolute right-1 top-1/2 -translate-y-1/2', search === '' && 'invisible']}
        >
          <X class="h-4" />
        </button>
      </div>
    </div>

    <!-- Champ list -->
    <div class="flex flex-wrap justify-center gap-x-4 gap-y-2 px-2">
      {#each filteredChamps as champ (champ.key)}
        <a href="/champs/{champ.name}" class="group rounded-lg">
          <img
            src="https://cdn.communitydragon.org/{champ.version}/champion/{champ.key}/square"
            alt="{champ.name} tile"
            width={100}
            class="rounded-lg transition-transform group-hover:scale-105"
          />

          <!-- TODO: nunu name too long -->
          <div class="text-center">{champ.name}</div>
        </a>
      {/each}
    </div>
  </div>
{/if}
