<script lang="ts">
  import {
    LayoutDashboard, ArrowUpDown, Tag, Database, Camera, PiggyBank,
    Settings, Info, Calculator, RefreshCw, Sun, Moon,
  } from '@lucide/svelte';
  import { currentPage, theme } from '../lib/stores';
  import { api } from '../lib/api';
  import type { Page } from '../lib/types';

  function toggleTheme() {
    const next = $theme === 'dark' ? 'light' : 'dark';
    theme.set(next);
    document.documentElement.setAttribute('data-theme', next);
    api.setSetting('theme', next).catch(() => {});
  }

  const mainNav: { label: string; page: Page; icon: typeof LayoutDashboard }[] = [
    { label: 'Dashboard',    page: 'dashboard',    icon: LayoutDashboard },
    { label: 'Transazioni',  page: 'transactions', icon: ArrowUpDown },
    { label: 'Categorie',    page: 'categories',   icon: Tag },
    { label: 'Dati',         page: 'export',       icon: Database },
    { label: 'Foto',         page: 'foto',         icon: Camera },
    { label: 'Obiettivi',    page: 'obiettivi',    icon: PiggyBank },
    { label: 'Ricorrenti',   page: 'ricorrenti',   icon: RefreshCw },
    { label: 'P.IVA',        page: 'piva',         icon: Calculator },
  ];

  const utilityNav: { label: string; page: Page; icon: typeof LayoutDashboard }[] = [
    { label: 'Impostazioni', page: 'settings', icon: Settings },
    { label: 'About',        page: 'about',    icon: Info },
  ];
</script>

<nav class="sidebar">
  <div class="logo">
    <svg viewBox="0 0 28 32" width="22" height="26" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round">
      <!-- Staff -->
      <line x1="8" y1="3" x2="8" y2="29" />
      <!-- Upper arm -->
      <line x1="8" y1="9" x2="22" y2="5" />
      <!-- Lower arm -->
      <line x1="8" y1="17" x2="22" y2="13" />
    </svg>
    <span>fehu</span>
  </div>

  {#each mainNav as item}
    <button class:active={$currentPage === item.page} onclick={() => currentPage.set(item.page)}>
      <svelte:component this={item.icon} size={16} />
      {item.label}
    </button>
  {/each}

  <div class="separator"></div>

  {#each utilityNav as item}
    <button class:active={$currentPage === item.page} onclick={() => currentPage.set(item.page)}>
      <svelte:component this={item.icon} size={16} />
      {item.label}
    </button>
  {/each}

  <div class="separator"></div>

  <button class="theme-toggle" onclick={toggleTheme}>
    {#if $theme === 'dark'}
      <Sun size={16} />
      <span>Tema chiaro</span>
    {:else}
      <Moon size={16} />
      <span>Tema scuro</span>
    {/if}
  </button>
</nav>

<style>
  .sidebar {
    width: 180px; min-height: 100vh;
    background: var(--sidebar-bg); color: var(--text);
    display: flex; flex-direction: column;
    padding: 1rem 0.75rem; gap: 0.25rem;
  }
  .theme-toggle {
    display: flex; align-items: center; gap: 0.5rem;
    padding: 0.55rem 0.75rem; border: none;
    background: transparent; color: var(--text-dim);
    cursor: pointer; border-radius: 6px;
    font-size: 0.82rem; text-align: left;
    transition: background 0.15s, color 0.15s;
    width: 100%;
  }
  .theme-toggle:hover { background: rgba(255,255,255,0.07); color: var(--text-muted); }
  .logo {
    display: flex; align-items: center; gap: 0.5rem;
    padding: 0.5rem 0.5rem 1.25rem;
    color: #a5b4fc;
  }
  .logo span {
    font-size: 1.15rem; font-weight: 700; letter-spacing: 0.06em;
  }
  button {
    display: flex; align-items: center; gap: 0.5rem;
    padding: 0.55rem 0.75rem; border: none;
    background: transparent; color: var(--text-muted);
    cursor: pointer; border-radius: 6px;
    font-size: 0.9rem; text-align: left;
    transition: background 0.15s, color 0.15s;
  }
  button:hover { background: rgba(255,255,255,0.07); color: var(--text); }
  button.active { background: rgba(99,102,241,0.18); color: var(--accent-lt); }
  .separator {
    margin: 0.5rem 0.5rem;
    border-top: 1px solid var(--border2);
    flex-shrink: 0;
  }
</style>
