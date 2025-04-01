<script lang="ts">
  import { onNavigate } from '$app/navigation'
  import type { Store } from '@tauri-apps/plugin-store'

  import { appState, themes } from '$lib'

  let store: Store | undefined
  let lockfilePath = $state.raw<string>()

  onNavigate(async () => {
    await store?.save()
  })
</script>

<form
  onsubmit={async () => {
    await store?.save()
  }}
  class="flex flex-col items-center gap-4"
>
  <!-- Options -->
  <div class="settings-grid grid grid-cols-[repeat(2,auto)] items-center gap-x-8 gap-y-4">
    <div>Lockfile path</div>
    <div>{lockfilePath}</div>

    <!-- Theme state changes are handled in Header.svelte -->
    <div>Theme</div>
    <div>
      {#each themes as t (t)}
        <button
          type="button"
          onclick={() => {
            appState.theme = t
          }}
          class={[
            'border border-r-0 px-2 py-1 capitalize first:rounded-l-lg last:rounded-r-lg last:border-r',
            t === appState.theme ? 'bg-gruvbox-aqua' : 'bg-gruvbox-bg dark:bg-gruvbox-dark-bg',
          ]}>{t}</button
        >
      {/each}
    </div>
  </div>

  <!-- Save button -->
  <div>
    <button class={['rounded-lg border px-2 py-1 transition-colors active:bg-gruvbox-aqua']}
      >Save</button
    >
  </div>
</form>
