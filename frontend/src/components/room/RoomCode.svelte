<script lang="ts">
  import Button from '../ui/Button.svelte';

  /** Shows the room code and copies it for sharing. */
  let { code, onerror }: { code: string; onerror: (message: string) => void } =
    $props();
  let copied = $state(false);

  async function copy(): Promise<void> {
    try {
      await navigator.clipboard.writeText(code);
      copied = true;
    } catch {
      onerror('Could not copy code');
    }
  }
</script>

<Button variant="quiet" size="small" onclick={copy} aria-label="Copy room code">
  <span class="code">{code}</span>
  <span aria-hidden="true">{copied ? '✓' : '⧉'}</span>
</Button>

<style>
  .code {
    letter-spacing: 0.12em;
  }
</style>
