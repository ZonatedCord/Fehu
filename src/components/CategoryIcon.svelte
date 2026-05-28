<script lang="ts">
  import {
    Utensils, Car, HeartPulse, Zap, Home, ShoppingCart,
    Briefcase, Gamepad2, Plane, BookOpen, Dumbbell, Coffee,
    Package, Shirt, Wrench, Music, MoreHorizontal,
  } from '@lucide/svelte';
  import type { Component } from 'svelte';

  let { name = 'package', size = 16 }: { name: string; size?: number } = $props();

  const map: Record<string, Component> = {
    'utensils': Utensils, 'car': Car, 'heart-pulse': HeartPulse,
    'zap': Zap, 'home': Home, 'shopping-cart': ShoppingCart,
    'briefcase': Briefcase, 'gamepad-2': Gamepad2, 'plane': Plane,
    'book-open': BookOpen, 'dumbbell': Dumbbell, 'coffee': Coffee,
    'package': Package, 'shirt': Shirt, 'wrench': Wrench,
    'music': Music, 'more-horizontal': MoreHorizontal,
  };

  const icon = $derived(map[name] ?? Package);
</script>

<svelte:component this={icon} {size} />
