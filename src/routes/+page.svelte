<script lang="ts">
  import { onMount } from 'svelte';
  import { fade } from 'svelte/transition';
  import { listen } from '@tauri-apps/api/event';
  import { api } from '../lib/api';
  import { currentPage, keyboardAction, theme, updateAvailable } from '../lib/stores';
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
  import Guida from '../pages/Guida.svelte';

  let showOnboarding = $state(false);
  let pendingUpdate = $state<any>(null);
  let updateInstalling = $state(false);

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

    // Check for updates silently after 3s
    setTimeout(async () => {
      try {
        const { check } = await import('@tauri-apps/plugin-updater');
        const update = await check();
        if (update) { pendingUpdate = update; updateAvailable.set(true); }
      } catch {}
    }, 3000);

    // Menu bar events
    let unlistenMenu: (() => void) | null = null;
    listen<string>('menu-action', (e) => {
      const id = e.payload;
      const navMap: Record<string, string> = {
        'nav-dashboard': 'dashboard', 'nav-transactions': 'transactions',
        'nav-categories': 'categories', 'nav-foto': 'foto',
        'nav-obiettivi': 'obiettivi', 'nav-ricorrenti': 'ricorrenti',
        'nav-piva': 'piva', 'nav-settings': 'settings', 'nav-export': 'export', 'nav-guida': 'guida',
      };
      if (navMap[id]) currentPage.set(navMap[id] as any);
      if (id === 'new-transaction') {
        currentPage.set('transactions');
        setTimeout(() => keyboardAction.set('new-transaction'), 50);
        setTimeout(() => keyboardAction.set(null), 100);
      }
      if (id === 'focus-search') {
        currentPage.set('transactions');
        setTimeout(() => keyboardAction.set('focus-search'), 50);
        setTimeout(() => keyboardAction.set(null), 100);
      }
      if (id === 'check-update') currentPage.set('settings');
    }).then(fn => { unlistenMenu = fn; });

    return () => {
      document.removeEventListener('keydown', onKey);
      unlistenMenu?.();
    };
  });

  async function installUpdate() {
    if (!pendingUpdate) return;
    updateInstalling = true;
    try { await pendingUpdate.downloadAndInstall(); } catch {}
    updateInstalling = false;
  }
</script>

{#if pendingUpdate}
  <div class="update-backdrop" transition:fade={{ duration: 150 }}>
    <div class="update-modal">
      <div class="update-icon">↑</div>
      <h2>Aggiornamento disponibile</h2>
      <p>Fehu <strong>v{pendingUpdate.version}</strong> è pronta per essere installata.</p>
      <div class="update-btns">
        <button class="upd-primary" onclick={installUpdate} disabled={updateInstalling}>
          {updateInstalling ? 'Installazione…' : 'Installa ora'}
        </button>
        <button class="upd-ghost" onclick={() => pendingUpdate = null}>Più tardi</button>
      </div>
    </div>
  </div>
{/if}

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
        {:else if $currentPage === 'guida'}<Guida />
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

  .update-backdrop {
    position: fixed; inset: 0; z-index: 200;
    display: flex; align-items: flex-start; justify-content: flex-end;
    padding: 3.5rem 1.5rem 0;
    pointer-events: none;
  }
  .update-modal {
    background: var(--bg-card); border: 1px solid var(--border);
    border-radius: 18px; padding: 1.5rem 1.75rem;
    width: 300px; box-shadow: 0 8px 32px rgba(0,0,0,0.18);
    pointer-events: all;
    display: flex; flex-direction: column; gap: 0.5rem;
  }
  .update-icon { font-size: 1.5rem; color: var(--accent); line-height: 1; }
  .update-modal h2 { margin: 0; font-size: 1rem; font-weight: 700; }
  .update-modal p { margin: 0; font-size: 0.875rem; color: var(--text-muted); }
  .update-btns { display: flex; gap: 0.5rem; margin-top: 0.5rem; }
  .upd-primary { background: var(--accent); color: #fff; border: none; padding: 0.5rem 1rem; border-radius: 10px; cursor: pointer; font-size: 0.875rem; font-weight: 600; flex: 1; }
  .upd-primary:disabled { opacity: 0.6; cursor: not-allowed; }
  .upd-ghost { background: var(--bg-elevated); border: none; color: var(--text-muted); padding: 0.5rem 0.75rem; border-radius: 10px; cursor: pointer; font-size: 0.875rem; }
</style>
