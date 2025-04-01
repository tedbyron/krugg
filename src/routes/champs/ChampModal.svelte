<script lang="ts">
  import X from '~icons/tabler/x'

  import { api, type ChampionShort } from '$lib'

  interface Props {
    selected?: ChampionShort
  }

  let { selected = $bindable() }: Props = $props()
  let dialog: HTMLDialogElement

  $effect(() => {
    if (selected !== undefined) {
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
    selected = undefined
  }}
  class="open:animate-zoom open:backdrop:animate-fade border bg-gruvbox-bg text-gruvbox-fg backdrop:bg-gruvbox-fg/80 dark:bg-gruvbox-dark-bg dark:text-gruvbox-dark-fg dark:backdrop:bg-gruvbox-dark-bg/80"
>
  <div class="container p-2">
    {#if api.champ !== undefined}
      <div>
        {JSON.stringify(api.champ, null, 2)}
      </div>
    {/if}

    <button
      type="button"
      onclick={() => {
        dialog.close()
      }}><X class="text-2xl" /></button
    >
  </div>
</dialog>

<style lang="postcss">
  dialog {
    scrollbar-gutter: stable both-edges;
  }
</style>
