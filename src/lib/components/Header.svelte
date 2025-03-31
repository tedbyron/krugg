<script lang="ts">
  import { onNavigate } from '$app/navigation'
  import { page } from '$app/state'
  import type { Store } from '@tauri-apps/plugin-store'
  import { onMount, type Component } from 'svelte'
  import type { MouseEventHandler, SvelteHTMLElements } from 'svelte/elements'
  import { connected } from 'tauri-plugin-lcu-api'
  import ArrowLeft from '~icons/tabler/arrow-left'
  import ArrowRight from '~icons/tabler/arrow-right'
  import CircleCheck from '~icons/tabler/circle-check-filled'
  import Circle from '~icons/tabler/circle-filled'
  import CircleX from '~icons/tabler/circle-x-filled'
  import Percentage50 from '~icons/tabler/percentage-50'
  import Settings from '~icons/tabler/settings-filled'

  import Tooltip from '$components/Tooltip.svelte'
  import { appData, lcu, type AppData } from '$lib'

  interface NavButton {
    name: string
    component?: Component<SvelteHTMLElements['svg']>
    class?: string
    disabled?: boolean
    onclick: MouseEventHandler<HTMLButtonElement>
  }

  const themes = [
    { name: 'system', component: Percentage50 },
    { name: 'light', component: Circle },
    { name: 'dark', component: Circle },
  ] as const satisfies { name: AppData['theme']; component: Component<SvelteHTMLElements['svg']> }[]

  let store: Store | undefined
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
      onclick: () => {
        window.navigation.back()
      },
    },
    {
      name: 'Forward',
      component: ArrowRight,
      disabled: !window.navigation.canGoForward,
      onclick: () => {
        window.navigation.forward()
      },
    },
  ])
  const rightNav: NavButton[] = $state([
    {
      name: 'Theme',
      component: Percentage50,
      onclick: () => {
        theme = themes[(themes.findIndex(({ name }) => name === theme.name) + 1) % themes.length]!
      },
    },
    {
      name: 'Settings',
      component: Settings,
      onclick: () => {
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
    store?.set('theme', theme.name).catch(console.error)
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

    store = await appData()
    const storedTheme = await store.get<AppData['theme']>('theme')
    if (storedTheme === undefined) {
      await store.set('theme', theme.name)
    } else {
      theme = themes.find(({ name }) => name === storedTheme)!
    }
  })

  // Conditionally disable back/forward buttons
  onNavigate(() => {
    leftNav[0]!.disabled = !window.navigation.canGoBack
    leftNav[1]!.disabled = !window.navigation.canGoForward
  })
</script>

{#snippet navButtons(buttons: NavButton[])}
  <nav class="flex gap-4">
    {#each buttons as btn (btn.name)}
      <button
        type="button"
        disabled={btn.disabled}
        onclick={btn.onclick}
        class={[
          'py-2 leading-none disabled:text-gruvbox-bg2 dark:disabled:text-gruvbox-dark-bg2',
          btn.class,
        ]}
      >
        {#if btn.component !== undefined}
          <btn.component />
        {:else}
          {btn.name}
        {/if}
      </button>
    {/each}
  </nav>
{/snippet}

<header class="flex items-center gap-4 border-b bg-gruvbox-bg px-3 dark:bg-gruvbox-dark-bg">
  <!-- Left nav buttons -->
  {@render navButtons(leftNav)}

  <!-- Page title -->
  <div class="ml-2 leading-none">{page.data.title}</div>

  <!-- LCU connection status icon -->
  <Tooltip text={lcuConnection.text} class="ml-auto py-2" tooltipClass="pt-1.5">
    <lcuConnection.component class={lcuConnection.class} />
  </Tooltip>

  <!-- Right nav buttons -->
  {@render navButtons(rightNav)}
</header>
