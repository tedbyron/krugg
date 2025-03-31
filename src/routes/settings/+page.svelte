<script lang="ts">
  import type { Store } from '@tauri-apps/plugin-store'
  import { onMount } from 'svelte'

  import { appData, type AppData } from '$lib'
  import { onNavigate } from '$app/navigation'

  const themes = ['system', 'light', 'dark'] as const satisfies AppData['theme'][]

  let store: Store | undefined
  let lockfilePath = $state.raw<string>()
  let theme = $state.raw<AppData['theme']>('system')

  $effect(() => {
    document.documentElement.classList.toggle(
      'dark',
      theme === 'dark' ||
        (theme === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches),
    )
    store?.set('theme', theme).catch(console.error)
  })

  onMount(async () => {
    store = await appData()
    lockfilePath = await store.get<string>('lockfile_path')
    // Always set by Header component on mount.
    theme = (await store.get<AppData['theme']>('theme'))!
  })

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
  <div class="settings-grid grid grid-cols-[repeat(2,auto)] items-center gap-4">
    <div>Lockfile path</div>
    <div>{lockfilePath}</div>

    <div>Theme</div>
    <div>
      {#each themes as t (t)}
        <button
          type="button"
          class={[
            'border border-r-0 bg-gruvbox-bg px-2 py-1 capitalize first:rounded-l-lg last:rounded-r-lg last:border-r dark:bg-gruvbox-dark-bg',
            t === theme &&
              'bg-gruvbox-aqua text-gruvbox-bg-h dark:bg-gruvbox-aqua dark:text-gruvbox-dark-bg-h',
          ]}
          onclick={() => {
            theme = t
          }}>{t}</button
        >
      {/each}
    </div>
  </div>

  <button class="rounded-lg border px-2 py-1">Save</button>
</form>
