<script lang="ts">
  import {
    Utensils, Car, HeartPulse, Zap, Home, ShoppingCart,
    Briefcase, Gamepad2, Plane, BookOpen, Dumbbell, Coffee,
    Package, Shirt, Wrench, Music, MoreHorizontal,
  } from '@lucide/svelte';
  import type { Component } from 'svelte';

  let { value = 'package', onselect }: { value: string; onselect: (name: string) => void } = $props();

  const icons: { name: string; label: string; component: Component }[] = [
    { name: 'utensils',        label: 'Cucina',        component: Utensils },
    { name: 'car',             label: 'Auto',          component: Car },
    { name: 'heart-pulse',     label: 'Salute',        component: HeartPulse },
    { name: 'zap',             label: 'Elettricità',   component: Zap },
    { name: 'home',            label: 'Casa',          component: Home },
    { name: 'shopping-cart',   label: 'Spesa',         component: ShoppingCart },
    { name: 'briefcase',       label: 'Lavoro',        component: Briefcase },
    { name: 'gamepad-2',       label: 'Svago',         component: Gamepad2 },
    { name: 'plane',           label: 'Viaggi',        component: Plane },
    { name: 'book-open',       label: 'Istruzione',    component: BookOpen },
    { name: 'dumbbell',        label: 'Sport',         component: Dumbbell },
    { name: 'coffee',          label: 'Bar',           component: Coffee },
    { name: 'package',         label: 'Altro',         component: Package },
    { name: 'shirt',           label: 'Abbigliamento', component: Shirt },
    { name: 'wrench',          label: 'Riparazioni',   component: Wrench },
    { name: 'music',           label: 'Musica',        component: Music },
    { name: 'more-horizontal', label: 'Generico',      component: MoreHorizontal },
  ];
</script>

<div class="picker">
  {#each icons as ic}
    {@const Icon = ic.component}
    <button
      type="button"
      class:selected={value === ic.name}
      onclick={() => onselect(ic.name)}
      title={ic.label}
    >
      <Icon size={18} />
    </button>
  {/each}
</div>

<style>
  .picker { display: flex; flex-wrap: wrap; gap: 0.25rem; }
  button {
    width: 36px; height: 36px; display: flex; align-items: center; justify-content: center;
    background: #0f0f1a; border: 1px solid #2e2e4e; border-radius: 6px;
    color: #888; cursor: pointer; transition: all 0.1s;
  }
  button:hover { border-color: #6366f1; color: #a5b4fc; }
  button.selected { border-color: #6366f1; background: #1e1e3e; color: #a5b4fc; }
</style>
