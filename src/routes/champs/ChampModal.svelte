<script lang="ts">
  import X from '~icons/tabler/x'

  import { api, type ChampionShort } from '$lib'

  interface Props {
    selectedChamp?: ChampionShort
  }

  let { selectedChamp: champShort = $bindable() }: Props = $props()
  let dialog: HTMLDialogElement

  $effect(() => {
    if (champShort !== undefined && api.champ !== undefined) {
      dialog.showModal()
    }
  })
</script>

<dialog
  bind:this={dialog}
  onclick={({ target }) => {
    if (target === dialog) {
      dialog.close()
    }
  }}
  onclose={() => {
    champShort = undefined
    api.champ = undefined
  }}
  class="h-[95dvh] max-h-[95dvh] w-[95dvw] max-w-screen-md border bg-gruvbox-bg text-gruvbox-fg backdrop:bg-gruvbox-fg/80 open:animate-zoom open:backdrop:animate-fade dark:bg-gruvbox-dark-bg dark:text-gruvbox-dark-fg dark:backdrop:bg-gruvbox-dark-bg/80"
>
  {#if champShort !== undefined && api.champ !== undefined}
    <div class="max-h-[calc(95dvh-4px)] max-w-[calc(95dvw-4px)]">
      <header
        class="sticky top-0 z-10 flex items-center border-b bg-gruvbox-bg px-2 dark:bg-gruvbox-dark-bg"
      >
        <button
          type="button"
          onclick={() => {
            dialog.close()
          }}
          class="ml-auto py-2"><X class="text-2xl" /></button
        >
      </header>

      <!-- Hero -->
      <div class="relative">
        <img
          src="https://cdn.communitydragon.org/{champShort.version}/champion/{champShort.key}/splash-art"
          alt={champShort.name}
          class="hero"
        />

        <div class="absolute top-1/3 ml-4 flex flex-col gap-4">
          <!-- Name and title -->
          <div class="flex items-end gap-4 leading-none">
            <h1 class="text-lg font-bold leading-none">{api.champ.name}</h1>
            <span class="text-base leading-none">&bull;</span>
            <h2 class="italic">{api.champ.title}</h2>
          </div>

          <!-- Lore -->
          <p class="max-w-prose">{api.champ.lore}</p>

          <div class="mt-4 flex gap-4">
            <!-- Roles -->
            <fieldset>
              <legend>Role{api.champ.tags.length > 1 ? 's' : ''}</legend>
              <span>{api.champ.tags.join(', ')}</span>
            </fieldset>

            <!-- Info -->
            <fieldset>
              <legend>Info</legend>

              {#each Object.entries(api.champ.info) as [key, val] (key)}
                <span>{key.charAt(0).toLocaleUpperCase() + key.slice(1)}:{val}</span>
              {/each}
            </fieldset>
          </div>
        </div>
      </div>
    </div>
  {/if}
</dialog>

<style lang="postcss">
  img.hero {
    mask-image: linear-gradient(to bottom, rgb(0, 0, 0, 0.75), transparent);
  }

  fieldset {
    @apply flex justify-center gap-2 rounded-lg border px-3 pb-1.5;
  }

  legend {
    @apply px-1;
  }
</style>
