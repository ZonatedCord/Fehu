<script lang="ts">
  import { LayoutDashboard, ArrowUpDown, Tag, Database, Camera, PiggyBank } from '@lucide/svelte';
  import { currentPage } from '../lib/stores';
  import type { Page } from '../lib/types';

  const nav: { label: string; page: Page; icon: typeof LayoutDashboard }[] = [
    { label: 'Dashboard',    page: 'dashboard',    icon: LayoutDashboard },
    { label: 'Transazioni', page: 'transactions', icon: ArrowUpDown },
    { label: 'Categorie',   page: 'categories',   icon: Tag },
    { label: 'Dati',         page: 'export',       icon: Database },
    { label: 'Foto',         page: 'foto',         icon: Camera },
    { label: 'Obiettivi',    page: 'obiettivi',    icon: PiggyBank },
  ];
</script>

<nav class="sidebar">
  <div class="logo">ᚠ fehu</div>
  {#each nav as item}
    <button class:active={$currentPage === item.page} onclick={() => currentPage.set(item.page)}>
      <svelte:component this={item.icon} size={16} />
      {item.label}
    </button>
  {/each}
</nav>

<style>
  .sidebar {
    width: 180px; min-height: 100vh;
    background: #111; color: #eee;
    display: flex; flex-direction: column;
    padding: 1rem 0.75rem; gap: 0.25rem;
  }
  .logo {
    font-size: 1.25rem; font-weight: 700;
    padding: 0.5rem 0.5rem 1.25rem;
    letter-spacing: 0.05em; color: #a5b4fc;
  }
  button {
    display: flex; align-items: center; gap: 0.5rem;
    padding: 0.55rem 0.75rem; border: none;
    background: transparent; color: #ccc;
    cursor: pointer; border-radius: 6px;
    font-size: 0.9rem; text-align: left;
    transition: background 0.15s;
  }
  button:hover { background: #1e1e2e; }
  button.active { background: #1e1e2e; color: #a5b4fc; }
</style>
