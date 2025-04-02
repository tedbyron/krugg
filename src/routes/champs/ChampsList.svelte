<script lang="ts">
  import { getChampion, type ChampionShort, api } from '$lib'
  import X from '~icons/tabler/x'

  interface Props {
    champs: ChampionShort[]
    search: string
    selectedChamp?: ChampionShort
  }

  let { champs, search = $bindable(), selectedChamp = $bindable() }: Props = $props()
</script>

<!-- TODO: hover preload splash art -->

<div class="container flex flex-col gap-2">
  <div class="relative mx-auto w-1/2 max-w-md">
    <!-- Search input -->
    <input
      type="text"
      autocapitalize="words"
      placeholder="Search"
      bind:value={search}
      class="w-full rounded-lg border bg-gruvbox-bg py-1 pl-2 pr-6 text-center leading-none text-gruvbox-fg placeholder:text-center placeholder:text-gruvbox-gray dark:bg-gruvbox-dark-bg dark:text-gruvbox-dark-gray dark:placeholder:text-gruvbox-dark-gray"
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

  <!-- Champ list -->
  <div class="flex flex-wrap justify-center gap-x-4 gap-y-2 px-2">
    {#each champs as champ (champ.id)}
      <button
        type="button"
        onclick={async () => {
          selectedChamp = champ
          if (api.champ?.id !== selectedChamp.id) {
            await getChampion(selectedChamp.id)
          }
        }}
        class="group flex flex-col items-center gap-0.5 rounded-lg"
      >
        <div
          class="max-w-[100px] overflow-hidden rounded-lg transition-transform group-hover:scale-105"
        >
          <img
            src="https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-icons/{champ.key}.png"
            alt="{champ.name} tile"
            width={100}
            class="scale-[1.15]"
          />
        </div>

        <!-- TODO: nunu name too long -->
        <div class="text-center">{champ.name}</div>
      </button>
    {/each}
  </div>
</div>
