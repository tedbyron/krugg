<script lang="ts">
  import type { Snippet } from 'svelte'
  import type { ClassValue } from 'svelte/elements'

  interface Props {
    class?: ClassValue
    tooltipClass?: ClassValue
    target: Snippet
    children: Snippet
  }

  const { class: className, tooltipClass, target, children }: Props = $props()
  const buttonId = crypto.randomUUID()
  const divId = crypto.randomUUID()

  let tooltip: HTMLDivElement
</script>

<button
  type="button"
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
  {@render target()}
</button>

<div
  bind:this={tooltip}
  id={divId}
  anchor={buttonId}
  popover
  class={[
    'absolute bottom-[anchor(center)] m-0 bg-gruvbox-bg text-gruvbox-fg dark:bg-gruvbox-dark-bg dark:text-gruvbox-dark-fg',
    tooltipClass,
  ]}
>
  {@render children()}
</div>
