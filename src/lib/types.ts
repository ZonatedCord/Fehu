export type TxType = 'income' | 'expense';

export interface Category {
  id: number;
  name: string;
  color: string;
  icon: string;
  budget_limit: number | null;
}

export interface BudgetAlert {
  category_id: number;
  category_name: string;
  budget_limit: number;
  spent: number;
}

export interface Transaction {
  id: number;
  amount: number;
  type: TxType;
  category_id: number | null;
  category_name: string | null;
  date: string;
  description: string;
  notes: string;
  source: string;
  metodo: string;
  currency: string;
  created_at: string;
  attachment_count: number;
}

export interface TransactionInput {
  amount: number;
  type: TxType;
  category_id: number | null;
  date: string;
  description: string;
  notes: string;
  metodo: string;
  currency: string;
}

export interface MonthlySummary {
  month: string;
  income: number;
  expense: number;
}

export interface CategorySummary {
  category_id: number | null;
  category_name: string | null;
  color: string | null;
  total: number;
}

export interface DashboardStats {
  total_income: number;
  total_expense: number;
  monthly: MonthlySummary[];
  by_category: CategorySummary[];
}

export interface ReceiptData {
  importo: number | null;
  data: string | null;
  descrizione: string | null;
  categoria: string | null;
  categoria_source: string | null;
  metodo: string | null;
}

export interface Goal {
  id: number;
  name: string;
  target: number;
  saved: number;
  color: string;
  icon: string;
  created_at: string;
}

export interface RecurringTemplate {
  id: number;
  description: string;
  amount: number;
  type: TxType;
  category_id: number | null;
  category_name: string | null;
  metodo: string;
  notes: string;
  frequency: string;
  next_date: string;
  active: boolean;
  created_at: string;
}

export interface RecurringInput {
  description: string;
  amount: number;
  type: TxType;
  category_id: number | null;
  metodo: string;
  notes: string;
  frequency: string;
  next_date: string;
}

export type Page = 'dashboard' | 'transactions' | 'categories' | 'export' | 'foto' | 'obiettivi' | 'settings' | 'about' | 'piva' | 'ricorrenti' | 'guida';

export interface AppSettings {
  ollama_url: string;
  tesseract_path: string;
  currency_symbol: string;
  onboarded: string;
  theme: string;
  telegram_token?: string;
}

export interface DepsStatus {
  tesseract: boolean;
  ollama: boolean;
  tesseract_version: string | null;
}

export interface BalanceAdjustment {
  id: number;
  metodo: string;
  amount: number;
  note: string;
  date: string;
  created_at: string;
}

export interface TransactionFile {
  id: number;
  transaction_id: number;
  file_name: string;
  file_path: string;
  created_at: string;
}

export interface PatrimonioStats {
  saldo_contanti: number;
  saldo_carta: number;
  totale: number;
}
