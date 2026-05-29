export type TxType = 'income' | 'expense';

export interface Category {
  id: number;
  name: string;
  color: string;
  icon: string;
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
  created_at: string;
}

export interface TransactionInput {
  amount: number;
  type: TxType;
  category_id: number | null;
  date: string;
  description: string;
  notes: string;
  metodo: string;
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

export type Page = 'dashboard' | 'transactions' | 'categories' | 'export' | 'foto' | 'obiettivi' | 'settings' | 'about' | 'piva';

export interface AppSettings {
  ollama_url: string;
  tesseract_path: string;
  currency_symbol: string;
  onboarded: string;
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
