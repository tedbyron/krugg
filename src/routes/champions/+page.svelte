<script lang="ts">
  import { champions, type ChampionShort } from '$lib'

  let search = $state('')
  let filteredChamps = $state<[string, ChampionShort][] | undefined>()

  const sortChamps = (champs: typeof $champions) => {
    if (champs !== undefined) {
      return Object.entries(champs).toSorted(([a], [b]) => a.localeCompare(b))
    } else {
      return undefined
    }
  }

  // pub fn search_champion(&self, name: &str) -> &ChampionShort {
  //     if self.champions.contains_key(name) {
  //         &self.champions[name]
  //     } else {
  //         let mut d_min = usize::MAX;
  //         let mut closest_champ: &ChampionShort = &self.champions["Annie"];

  //         let mut d_min_prefix = usize::MAX;
  //         let mut closest_champ_prefix: Option<&ChampionShort> = None;

  //         for value in self.champions.values() {
  //             let query_cmp = name.to_lowercase();
  //             let champ_cmp = value.name.to_lowercase();
  //             // Prefer matches where the query is an exact prefix.
  //             let d = levenshtein::levenshtein(query_cmp.as_str(), champ_cmp.as_str());
  //             if champ_cmp.starts_with(&query_cmp) {
  //                 if d <= d_min_prefix {
  //                     d_min_prefix = d;
  //                     closest_champ_prefix = Some(value);
  //                 }
  //             } else if d <= d_min {
  //                 d_min = d;
  //                 closest_champ = value;
  //             }
  //         }

  //         closest_champ_prefix.unwrap_or(closest_champ)
  //     }
  // }

  $effect(() => {
    if ($champions !== undefined && search !== '') {
      filteredChamps = sortChamps($champions)!.filter(([, champion]) =>
        champion.name.toLowerCase().includes(search.toLowerCase()),
      )
    } else {
      filteredChamps = sortChamps($champions)
    }
  })
</script>

{#if filteredChamps !== undefined}
  <div class="flex justify-center">
    <input
      type="text"
      name="search"
      autocapitalize="words"
      placeholder="Search"
      aria-label="Search champions"
      bind:value={search}
      class="w-1/2 rounded-lg border border-zinc-50 bg-zinc-900 px-2 py-1 text-center text-zinc-50 placeholder:text-center"
    />
  </div>

  <div class="flex flex-wrap justify-center gap-4 overflow-auto p-4">
    {#each filteredChamps as [key, champion] (key)}
      <a href="./{champion.name}" class="">
        <img
          src="https://cdn.communitydragon.org/{champion.version}/champion/{champion.key}/square"
          alt="{champion.name} tile"
          width={128}
          class="inline-block rounded-lg"
        />
        <div class="text-center">{champion.name}</div>
      </a>
    {/each}
  </div>
{/if}
