<script lang="ts">
  import { onMount } from 'svelte';
  import { fade } from 'svelte/transition';
  import { api } from '../lib/api';
  import { currentPage, keyboardAction, theme } from '../lib/stores';
  import Sidebar from '../components/Sidebar.svelte';
  import Onboarding from '../components/Onboarding.svelte';
  import Dashboard from '../pages/Dashboard.svelte';
  import Transactions from '../pages/Transactions.svelte';
  import Categories from '../pages/Categories.svelte';
  import Export from '../pages/Export.svelte';
  import Foto from '../pages/Foto.svelte';
  import Obiettivi from '../pages/Obiettivi.svelte';
  import Settings from '../pages/Settings.svelte';
  import About from '../pages/About.svelte';
  import PIva from '../pages/PIva.svelte';
  import Ricorrenti from '../pages/Ricorrenti.svelte';

  let showOnboarding = $state(false);

  onMount(() => {
    api.getSettings().then(s => {
      if (s.onboarded !== 'true') showOnboarding = true;
      const t = (s as any).theme ?? 'dark';
      theme.set(t);
      document.documentElement.setAttribute('data-theme', t);
    }).catch(() => { showOnboarding = true; });

    function onKey(e: KeyboardEvent) {
      const mod = e.metaKey || e.ctrlKey;
      if (!mod) return;
      if (e.key === 'n' || e.key === 'N') {
        e.preventDefault();
        currentPage.set('transactions');
        setTimeout(() => keyboardAction.set('new-transaction'), 50);
        setTimeout(() => keyboardAction.set(null), 100);
      } else if (e.key === 'f' || e.key === 'F') {
        e.preventDefault();
        currentPage.set('transactions');
        setTimeout(() => keyboardAction.set('focus-search'), 50);
        setTimeout(() => keyboardAction.set(null), 100);
      } else if (e.key === ',') {
        e.preventDefault();
        currentPage.set('settings');
      }
    }
    document.addEventListener('keydown', onKey);
    return () => document.removeEventListener('keydown', onKey);
  });
</script>

{#if showOnboarding}
  <Onboarding onComplete={() => { showOnboarding = false; }} />
{/if}

<div class="layout">
  <Sidebar />
  <main class="content">
    {#key $currentPage}
      <div transition:fade={{ duration: 110 }}>
        {#if $currentPage === 'dashboard'}<Dashboard />
        {:else if $currentPage === 'transactions'}<Transactions />
        {:else if $currentPage === 'categories'}<Categories />
        {:else if $currentPage === 'export'}<Export />
        {:else if $currentPage === 'foto'}<Foto />
        {:else if $currentPage === 'obiettivi'}<Obiettivi />
        {:else if $currentPage === 'settings'}<Settings />
        {:else if $currentPage === 'about'}<About />
        {:else if $currentPage === 'piva'}<PIva />
        {:else if $currentPage === 'ricorrenti'}<Ricorrenti />
        {/if}
      </div>
    {/key}
  </main>
</div>

<style>
  :global(:root) {
    --bg-base:     #0f0f1a;
    --bg-card:     #1a1a2e;
    --bg-card2:    #111120;
    --bg-elevated: #252540;
    --bg-input:    #0f0f1a;
    --border:      #2e2e4e;
    --border2:     #1e1e2e;
    --text:        #e0e0f0;
    --text-muted:  #9098b0;
    --text-dim:    #565878;
    --accent:      #6366f1;
    --accent-lt:   #a5b4fc;
    --income:      #4ade80;
    --expense:     #f87171;
    --sidebar-bg:    #0c0c18;
    --sidebar-hover: rgba(255,255,255,0.07);
    --sidebar-logo:  #a5b4fc;
  }
  :global([data-theme="light"]) {
    --bg-base:     #e8eaf4;
    --bg-card:     #ffffff;
    --bg-card2:    #f4f5fb;
    --bg-elevated: #dde0f0;
    --bg-input:    #f0f2fa;
    --border:      #b8bdd6;
    --border2:     #d0d4e8;
    --text:        #0f172a;
    --text-muted:  #374151;
    --text-dim:    #6b7280;
    --accent:      #4f46e5;
    --accent-lt:   #6366f1;
    --income:      #15803d;
    --expense:     #b91c1c;
    --sidebar-bg:   #d4d8ef;
    --sidebar-hover: rgba(0,0,0,0.07);
    --sidebar-logo:  #4f46e5;
  }
  :global(*, *::before, *::after) { box-sizing: border-box; }
  :global(body) {
    margin: 0; font-family: system-ui, -apple-system, sans-serif;
    background: var(--bg-base); color: var(--text);
    transition: background 0.2s, color 0.2s;
  }
  /* Global modern defaults */
  :global(input), :global(select), :global(textarea) {
    border-radius: 10px !important;
  }
  :global(.btn-primary), :global(.btn-ghost), :global(.btn-ghost-small) {
    border-radius: 10px !important;
  }
  /* Center all page components within the content area */
  :global(.page) { margin: 0 auto; }
  .layout { display: flex; min-height: 100vh; padding: 0.75rem; gap: 0.75rem; background: var(--bg-base); }
  .content { flex: 1; padding: 2rem 2.5rem; overflow-y: auto; background: var(--bg-card); border-radius: 16px; box-shadow: 0 1px 4px rgba(0,0,0,0.07); }
</style>
