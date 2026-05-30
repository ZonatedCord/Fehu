import { writable } from 'svelte/store';
import type { Page } from './types';

export const currentPage = writable<Page>('dashboard');

// Keyboard action store: emits 'new-transaction' | 'focus-search' | null
export const keyboardAction = writable<string | null>(null);

export const theme = writable<'dark' | 'light'>('dark');
