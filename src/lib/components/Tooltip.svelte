<script lang="ts">
  import type { Snippet } from 'svelte'
  import type { ClassValue } from 'svelte/elements'

  interface Props {
    text: string
    class?: ClassValue
    children: Snippet
  }

  const { text, class: className, children }: Props = $props()
  const buttonId = crypto.randomUUID()
  const divId = crypto.randomUUID()

  let tooltip: HTMLDivElement
</script>

<button
  id={buttonId}
  popovertarget={divId}
  onclick={(evt) => {
    evt.preventDefault()
  }}
  onmouseover={() => {
    tooltip.showPopover()
  }}
  onmouseleave={() => {
    tooltip.hidePopover()
  }}
  onfocus={() => {
    tooltip.showPopover()
  }}
  onblur={() => {
    tooltip.hidePopover()
  }}
  class={['cursor-default', className]}
>
  {@render children()}
</button>

<div
  bind:this={tooltip}
  id={divId}
  anchor={buttonId}
  popover
  class="absolute bottom-[anchor(center)] m-0 bg-gruvbox-bg dark:bg-gruvbox-dark-bg"
>
  {text}
</div>
