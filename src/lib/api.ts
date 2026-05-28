import { invoke } from '@tauri-apps/api/core';
import type { Category, DashboardStats, Transaction, TransactionInput } from './types';

export const api = {
  listCategories: () => invoke<Category[]>('list_categories'),
  createCategory: (name: string, color: string, icon: string) =>
    invoke<Category>('create_category', { name, color, icon }),
  updateCategory: (id: number, name: string, color: string, icon: string) =>
    invoke<void>('update_category', { id, name, color, icon }),
  deleteCategory: (id: number) => invoke<void>('delete_category', { id }),

  listTransactions: (filters?: { start_date?: string; end_date?: string; category_id?: number }) =>
    invoke<Transaction[]>('list_transactions', filters ?? {}),
  createTransaction: (input: TransactionInput) =>
    invoke<Transaction>('create_transaction', { input }),
  updateTransaction: (id: number, input: TransactionInput) =>
    invoke<void>('update_transaction', { id, input }),
  deleteTransaction: (id: number) => invoke<void>('delete_transaction', { id }),

  getDashboardStats: (filters?: { start_date?: string; end_date?: string }) =>
    invoke<DashboardStats>('get_dashboard_stats', filters ?? {}),
  exportCsv: (filters?: { start_date?: string; end_date?: string }) =>
    invoke<string>('export_csv', filters ?? {}),
  exportXlsx: (filePath: string, filters?: { start_date?: string; end_date?: string }) =>
    invoke<void>('export_xlsx', { filePath, ...filters }),
};
