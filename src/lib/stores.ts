import { writable } from 'svelte/store';
import type { Page } from './types';

export const currentPage = writable<Page>('dashboard');
