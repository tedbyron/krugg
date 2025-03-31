<script lang="ts">
  import type { PageProps } from './$types'

  const { data }: PageProps = $props()
</script>

{#if data.champ !== undefined}
  {JSON.stringify(data.champ, null, 2)}
{/if}
