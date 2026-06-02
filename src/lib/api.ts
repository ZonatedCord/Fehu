import { invoke } from '@tauri-apps/api/core';
import type { AppSettings, BalanceAdjustment, BudgetAlert, Category, DashboardStats, DepsStatus, Goal, PatrimonioStats, RecurringInput, RecurringTemplate, Transaction, TransactionFile, TransactionInput } from './types';

export const api = {
  listCategories: () => invoke<Category[]>('list_categories'),
  createCategory: (name: string, color: string, icon: string, budget_limit?: number | null) =>
    invoke<Category>('create_category', { name, color, icon, budget_limit: budget_limit ?? null }),
  updateCategory: (id: number, name: string, color: string, icon: string, budget_limit?: number | null) =>
    invoke<void>('update_category', { id, name, color, icon, budget_limit: budget_limit ?? null }),
  deleteCategory: (id: number) => invoke<void>('delete_category', { id }),
  getBudgetAlerts: (month: string) => invoke<BudgetAlert[]>('get_budget_alerts', { month }),

  listTransactions: (filters?: { start_date?: string; end_date?: string; category_id?: number; search_text?: string }) =>
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

  listGoals: () => invoke<Goal[]>('list_goals'),
  createGoal: (name: string, target: number, color: string) =>
    invoke<Goal>('create_goal', { name, target, color }),
  updateGoal: (id: number, name: string, target: number, color: string) =>
    invoke<void>('update_goal', { id, name, target, color }),
  updateGoalSaved: (id: number, saved: number) =>
    invoke<void>('update_goal_saved', { id, saved }),
  deleteGoal: (id: number) => invoke<void>('delete_goal', { id }),
  contributeToGoal: (goalId: number, amount: number, metodo: string, date: string) =>
    invoke<Goal>('contribute_to_goal', { goalId, amount, metodo, date }),

  getPatrimonio: () => invoke<PatrimonioStats>('get_patrimonio'),
  listBalanceAdjustments: () => invoke<BalanceAdjustment[]>('list_balance_adjustments'),
  createBalanceAdjustment: (metodo: string, amount: number, note: string, date: string) =>
    invoke<BalanceAdjustment>('create_balance_adjustment', { metodo, amount, note, date }),
  deleteBalanceAdjustment: (id: number) => invoke<void>('delete_balance_adjustment', { id }),

  getSettings: () => invoke<AppSettings>('get_settings'),
  setSetting: (key: string, value: string) => invoke<void>('set_setting', { key, value }),

  checkDependencies: () => invoke<DepsStatus>('check_dependencies'),
  installDependency: (dep: string) => invoke<{ success: boolean; output: string }>('install_dependency', { dep }),

  attachFile: (transactionId: number, sourcePath: string) =>
    invoke<TransactionFile>('attach_file', { transactionId, sourcePath }),
  listAttachments: (transactionId: number) =>
    invoke<TransactionFile[]>('list_attachments', { transactionId }),
  deleteAttachment: (id: number) => invoke<void>('delete_attachment', { id }),

  startTelegramBot: (token: string) => invoke<string>('start_telegram_bot', { token }),
  stopTelegramBot: () => invoke<void>('stop_telegram_bot'),
  getTelegramStatus: () => invoke<boolean>('get_telegram_status'),

  exportDatabase: (targetPath: string) => invoke<void>('export_database', { targetPath }),
  restoreDatabase: (sourcePath: string) => invoke<void>('restore_database', { sourcePath }),

  listRecurring: () => invoke<RecurringTemplate[]>('list_recurring'),
  createRecurring: (input: RecurringInput) => invoke<RecurringTemplate>('create_recurring', { input }),
  updateRecurring: (id: number, input: RecurringInput) => invoke<void>('update_recurring', { id, input }),
  deleteRecurring: (id: number) => invoke<void>('delete_recurring', { id }),
  toggleRecurring: (id: number) => invoke<boolean>('toggle_recurring', { id }),
  checkAndInsertRecurring: () => invoke<number>('check_and_insert_recurring'),
};
