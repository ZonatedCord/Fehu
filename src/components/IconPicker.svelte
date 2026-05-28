<script lang="ts">
  import {
    Utensils, Car, HeartPulse, Zap, Home, ShoppingCart,
    Briefcase, Gamepad2, Plane, BookOpen, Dumbbell, Coffee,
    Package, Shirt, Wrench, Music, MoreHorizontal,
  } from '@lucide/svelte';
  import type { Component } from 'svelte';

  let { value = 'package', onselect }: { value: string; onselect: (name: string) => void } = $props();

  const icons: { name: string; component: Component }[] = [
    { name: 'utensils',        component: Utensils },
    { name: 'car',             component: Car },
    { name: 'heart-pulse',     component: HeartPulse },
    { name: 'zap',             component: Zap },
    { name: 'home',            component: Home },
    { name: 'shopping-cart',   component: ShoppingCart },
    { name: 'briefcase',       component: Briefcase },
    { name: 'gamepad-2',       component: Gamepad2 },
    { name: 'plane',           component: Plane },
    { name: 'book-open',       component: BookOpen },
    { name: 'dumbbell',        component: Dumbbell },
    { name: 'coffee',          component: Coffee },
    { name: 'package',         component: Package },
    { name: 'shirt',           component: Shirt },
    { name: 'wrench',          component: Wrench },
    { name: 'music',           component: Music },
    { name: 'more-horizontal', component: MoreHorizontal },
  ];
</script>

<div class="picker">
  {#each icons as ic}
    {@const Icon = ic.component}
    <button
      type="button"
      class:selected={value === ic.name}
      onclick={() => onselect(ic.name)}
      title={ic.name}
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
