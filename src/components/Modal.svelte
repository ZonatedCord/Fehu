<script lang="ts">
  import type { Snippet } from 'svelte';
  let { open = false, title = '', onclose, children }: {
    open: boolean; title: string; onclose: () => void; children: Snippet;
  } = $props();
</script>

{#if open}
  <div class="backdrop" role="dialog" aria-modal="true">
    <div class="modal">
      <div class="header">
        <h2>{title}</h2>
        <button onclick={onclose}>✕</button>
      </div>
      <div class="body">{@render children()}</div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed; inset: 0; background: rgba(0,0,0,0.6);
    display: flex; align-items: center; justify-content: center; z-index: 100;
  }
  .modal {
    background: var(--bg-card); border-radius: 10px;
    min-width: 380px; max-width: 520px; width: 90%;
    box-shadow: 0 20px 60px rgba(0,0,0,0.5);
  }
  .header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 1rem 1.25rem 0.75rem; border-bottom: 1px solid #2e2e4e;
  }
  h2 { margin: 0; font-size: 1rem; color: var(--text); }
  .header button { background: none; border: none; color: var(--text-muted); cursor: pointer; font-size: 1rem; }
  .body { padding: 1.25rem; }
</style>
