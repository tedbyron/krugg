<script lang="ts">
  import { onNavigate } from '$app/navigation'
  import type { Store } from '@tauri-apps/plugin-store'
  import { onMount, type Component } from 'svelte'
  import type { MouseEventHandler, SvelteHTMLElements } from 'svelte/elements'
  import { connected } from 'tauri-plugin-lcu-api'
  import ArrowLeft from '~icons/tabler/arrow-left'
  import ArrowRight from '~icons/tabler/arrow-right'
  import CircleCheck from '~icons/tabler/circle-check-filled'
  import CircleX from '~icons/tabler/circle-x-filled'
  import Settings from '~icons/tabler/settings-filled'
  import LightDark from '~icons/tabler/circle-filled'
  import System from '~icons/tabler/percentage-50'

  import Tooltip from '$components/Tooltip.svelte'
  import { lcu, loadAppData } from '$lib'

  interface NavButton {
    name: string
    component?: Component<SvelteHTMLElements['svg']>
    class?: string
    disabled?: boolean
    onclick: MouseEventHandler<HTMLButtonElement>
  }

  const themes = [
    { name: 'system', component: System },
    { name: 'light', component: LightDark },
    { name: 'dark', component: LightDark },
  ] as const

  let appData: Store | undefined
  let theme = $state.raw<(typeof themes)[number]>(themes[0])
  let lcuConnection = $state.raw({
    component: CircleX,
    text: 'League client disconnected',
    class: 'text-gruvbox-red',
  })
  const leftNav: NavButton[] = $state([
    {
      name: 'Back',
      component: ArrowLeft,
      disabled: !window.navigation.canGoBack,
      onclick: (evt) => {
        evt.preventDefault()
        window.navigation.back()
      },
    },
    {
      name: 'Forward',
      component: ArrowRight,
      disabled: !window.navigation.canGoForward,
      onclick: (evt) => {
        evt.preventDefault()
        window.navigation.forward()
      },
    },
  ])
  const rightNav: NavButton[] = $state([
    {
      name: 'Theme',
      component: System,
      onclick: (evt) => {
        evt.preventDefault()
        theme = themes[(themes.findIndex(({ name }) => name === theme.name) + 1) % themes.length]!
      },
    },
    {
      name: 'Settings',
      component: Settings,
      onclick: (evt) => {
        evt.preventDefault()
        window.location.href = '/settings'
      },
    },
  ])

  // Update theme
  $effect(() => {
    document.documentElement.classList.toggle(
      'dark',
      theme.name === 'dark' ||
        (theme.name === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches),
    )
    rightNav[0]!.component = theme.component
    appData?.set('theme', theme.name).catch(console.error)
  })

  // Update LCU connection status icon
  $effect(() => {
    if (lcu.connected) {
      lcuConnection = {
        component: CircleCheck,
        text: 'League client connected',
        class: 'text-gruvbox-green',
      }
    } else {
      lcuConnection = {
        component: CircleX,
        text: 'League client disconnected',
        class: 'text-gruvbox-red',
      }
    }
  })

  onMount(async () => {
    lcu.connected = await connected()

    appData = await loadAppData()
    const storedTheme = await appData.get<(typeof themes)[number]['name']>('theme')
    if (storedTheme !== undefined) {
      theme = themes.find(({ name }) => name === storedTheme)!
    }
  })

  onNavigate(() => {
    leftNav[0]!.disabled = !window.navigation.canGoBack
    leftNav[1]!.disabled = !window.navigation.canGoForward
  })
</script>

{#snippet navButtons(list: NavButton[])}
  <nav class="flex gap-2">
    {#each list as btn (btn.name)}
      <button
        disabled={btn.disabled}
        onclick={btn.onclick}
        class={[
          'py-2 leading-none disabled:text-gruvbox-bg2 dark:disabled:text-gruvbox-dark-bg2',
          btn.class,
        ]}
      >
        {#if btn.component !== undefined}
          <btn.component class="h-4" />
        {:else}
          {btn.name}
        {/if}
      </button>
    {/each}
  </nav>
{/snippet}

<header
  class="sticky top-0 flex items-center gap-2 border-b bg-gruvbox-bg px-2 dark:bg-gruvbox-dark-bg"
>
  <!-- Left nav buttons -->
  {@render navButtons(leftNav)}

  <!-- LCU connection status icon -->
  <Tooltip text={lcuConnection.text} class="ml-auto py-2">
    <lcuConnection.component class={['h-4', lcuConnection.class]} />
  </Tooltip>

  <!-- Right nav buttons -->
  {@render navButtons(rightNav)}
</header>
