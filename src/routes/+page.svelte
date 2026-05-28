<script lang="ts">
  import { currentPage } from '../lib/stores';
  import Sidebar from '../components/Sidebar.svelte';
  import Dashboard from '../pages/Dashboard.svelte';
  import Transactions from '../pages/Transactions.svelte';
  import Categories from '../pages/Categories.svelte';
  import Export from '../pages/Export.svelte';
</script>

<div class="layout">
  <Sidebar />
  <main class="content">
    {#if $currentPage === 'dashboard'}<Dashboard />
    {:else if $currentPage === 'transactions'}<Transactions />
    {:else if $currentPage === 'categories'}<Categories />
    {:else if $currentPage === 'export'}<Export />
    {/if}
  </main>
</div>

<style>
  :global(*, *::before, *::after) { box-sizing: border-box; }
  :global(body) {
    margin: 0; font-family: system-ui, -apple-system, sans-serif;
    background: #0f0f1a; color: #e0e0f0;
  }
  .layout { display: flex; min-height: 100vh; }
  .content { flex: 1; padding: 1.5rem; overflow-y: auto; }
</style>
