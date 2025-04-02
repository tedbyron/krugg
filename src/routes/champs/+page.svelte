<script lang="ts">
  import { api, type ChampionShort } from '$lib'
  import ChampModal from './ChampModal.svelte'
  import ChampsList from './ChampsList.svelte'

  let search = $state.raw('')
  let selectedChamp = $state.raw<ChampionShort>()
  let filteredChamps = $state.raw<ChampionShort[]>()

  // Filter champs based on search input.
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
  <ChampsList champs={filteredChamps} bind:selectedChamp bind:search />
{/if}

<!-- Can't use [slug] routes without SSR, so using a modal -->
<ChampModal bind:selectedChamp />
