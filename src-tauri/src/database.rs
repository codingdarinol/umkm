use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use csv::ReaderBuilder;

#[derive(Debug, Serialize, Deserialize)]
pub struct Container {
    pub id: i64,
    pub name: String,
    pub created_at: String,
    pub is_default: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub id: i64,
    pub name: String,
    pub account_type: String,
    pub opening_balance: i64,
    pub container_id: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountBalance {
    pub id: i64,
    pub name: String,
    pub account_type: String,
    pub opening_balance: i64,
    pub balance: i64,
    pub container_id: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Category {
    pub name: String,
    pub category_type: String,
    pub is_default: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub id: i64,
    pub amount: i64,
    pub description: String,
    pub category: String,
    pub date: String,
    pub container_id: i64,
    pub account_id: i64,
    pub transfer_id: i64,
    pub transfer_account_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewTransaction {
    pub amount: i64,
    pub description: Option<String>,
    pub category: Option<String>,
    pub container_id: i64,
    pub account_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfitLossLine {
    pub category: String,
    pub total: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfitLossReport {
    pub start_date: String,
    pub end_date: String,
    pub income: Vec<ProfitLossLine>,
    pub expense: Vec<ProfitLossLine>,
    pub total_income: i64,
    pub total_expense: i64,
    pub net_income: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceSheetReport {
    pub as_of: String,
    pub assets: Vec<AccountBalance>,
    pub liabilities: Vec<AccountBalance>,
    pub equity: Vec<AccountBalance>,
    pub total_assets: i64,
    pub total_liabilities: i64,
    pub total_equity: i64,
}

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS containers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL,
                is_default INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;

        let container_count: i64 = conn.query_row("SELECT COUNT(*) FROM containers", [], |row| row.get(0))?;
        if container_count == 0 {
            let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            conn.execute(
                "INSERT INTO containers (name, created_at, is_default) VALUES (?1, ?2, 1)",
                ["Personal", &now],
            )?;
        }

        conn.execute(
            "CREATE TABLE IF NOT EXISTS transactions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                amount INTEGER NOT NULL,
                description TEXT NOT NULL,
                category TEXT NOT NULL,
                date TEXT NOT NULL,
                container_id INTEGER NOT NULL DEFAULT 1,
                account_id INTEGER,
                transfer_id INTEGER,
                transfer_account_id INTEGER,
                FOREIGN KEY (container_id) REFERENCES containers(id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS accounts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                account_type TEXT NOT NULL,
                opening_balance INTEGER NOT NULL DEFAULT 0,
                container_id INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                UNIQUE(name, container_id),
                FOREIGN KEY (container_id) REFERENCES containers(id) ON DELETE CASCADE
            )",
            [],
        )?;

        let has_container_id: Result<i64, _> = conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('transactions') WHERE name='container_id'",
            [],
            |row| row.get(0)
        );
        
        if let Ok(0) = has_container_id {
            conn.execute(
                "ALTER TABLE transactions ADD COLUMN container_id INTEGER NOT NULL DEFAULT 1",
                [],
            )?;
        }

        let has_account_id: Result<i64, _> = conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('transactions') WHERE name='account_id'",
            [],
            |row| row.get(0)
        );

        if let Ok(0) = has_account_id {
            conn.execute(
                "ALTER TABLE transactions ADD COLUMN account_id INTEGER",
                [],
            )?;
        }

        let has_transfer_id: Result<i64, _> = conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('transactions') WHERE name='transfer_id'",
            [],
            |row| row.get(0),
        );

        if let Ok(0) = has_transfer_id {
            conn.execute(
                "ALTER TABLE transactions ADD COLUMN transfer_id INTEGER",
                [],
            )?;
        }

        let has_transfer_account_id: Result<i64, _> = conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('transactions') WHERE name='transfer_account_id'",
            [],
            |row| row.get(0),
        );

        if let Ok(0) = has_transfer_account_id {
            conn.execute(
                "ALTER TABLE transactions ADD COLUMN transfer_account_id INTEGER",
                [],
            )?;
        }

        conn.execute(
            "CREATE TABLE IF NOT EXISTS categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                category_type TEXT NOT NULL DEFAULT 'expense',
                is_default INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;

        let has_category_type: Result<i64, _> = conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('categories') WHERE name='category_type'",
            [],
            |row| row.get(0),
        );

        if let Ok(0) = has_category_type {
            conn.execute(
                "ALTER TABLE categories ADD COLUMN category_type TEXT NOT NULL DEFAULT 'expense'",
                [],
            )?;
        }

        let count: i64 = conn.query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))?;
        if count == 0 {
            let defaults = vec![
                ("Food & Dining", "expense"),
                ("Transportation", "expense"),
                ("Shopping", "expense"),
                ("Entertainment", "expense"),
                ("Bills & Utilities", "expense"),
                ("Healthcare", "expense"),
                ("Income", "income"),
                ("Other", "expense"),
            ];
            for (category, category_type) in defaults {
                conn.execute(
                    "INSERT INTO categories (name, category_type, is_default) VALUES (?1, ?2, 1)",
                    [category, category_type],
                )?;
            }
        }

        conn.execute(
            "UPDATE categories SET category_type = 'expense' WHERE category_type IS NULL",
            [],
        )?;
        conn.execute(
            "UPDATE categories SET category_type = 'income' WHERE name = 'Income'",
            [],
        )?;

        Ok(Database {
            conn: Mutex::new(conn),
        })
    }

    pub fn add_transaction(&self, transaction: NewTransaction) -> Result<Transaction> {
        let conn = self.conn.lock().unwrap();
        let date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        let description = transaction.description.unwrap_or_else(|| "Untitled".to_string());
        let category = transaction.category.unwrap_or_else(|| "Other".to_string());
        
        conn.execute(
            "INSERT INTO transactions (amount, description, category, date, container_id, account_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            [
                &transaction.amount.to_string(),
                &description,
                &category,
                &date,
                &transaction.container_id.to_string(),
                &transaction.account_id.to_string(),
            ],
        )?;

        let id = conn.last_insert_rowid();
        
        Ok(Transaction {
            id,
            amount: transaction.amount,
            description,
            category,
            date,
            container_id: transaction.container_id,
            account_id: transaction.account_id,
            transfer_id: 0,
            transfer_account_id: 0,
        })
    }

    pub fn add_transfer(
        &self,
        container_id: i64,
        from_account_id: i64,
        to_account_id: i64,
        amount: i64,
        description: Option<String>,
    ) -> Result<i64> {
        if from_account_id == to_account_id {
            return Err(rusqlite::Error::InvalidParameterName(
                "Source and destination accounts must be different".to_string(),
            ));
        }
        if amount <= 0 {
            return Err(rusqlite::Error::InvalidParameterName(
                "Transfer amount must be positive".to_string(),
            ));
        }

        let conn = self.conn.lock().unwrap();
        let date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let description = description.unwrap_or_else(|| "Transfer".to_string());

        let transfer_id: i64 = conn.query_row(
            "SELECT COALESCE(MAX(transfer_id), 0) + 1 FROM transactions",
            [],
            |row| row.get(0),
        )?;

        let debit_amount = -amount.abs();
        let credit_amount = amount.abs();

        conn.execute(
            "INSERT INTO transactions (amount, description, category, date, container_id, account_id, transfer_id, transfer_account_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            [
                &debit_amount.to_string(),
                &description,
                "Transfer",
                &date,
                &container_id.to_string(),
                &from_account_id.to_string(),
                &transfer_id.to_string(),
                &to_account_id.to_string(),
            ],
        )?;

        conn.execute(
            "INSERT INTO transactions (amount, description, category, date, container_id, account_id, transfer_id, transfer_account_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            [
                &credit_amount.to_string(),
                &description,
                "Transfer",
                &date,
                &container_id.to_string(),
                &to_account_id.to_string(),
                &transfer_id.to_string(),
                &from_account_id.to_string(),
            ],
        )?;

        Ok(transfer_id)
    }

    pub fn get_transactions(&self, container_id: i64, limit: Option<i64>) -> Result<Vec<Transaction>> {
        let conn = self.conn.lock().unwrap();
        let query = match limit {
            Some(l) => format!("SELECT id, amount, description, category, date, container_id, COALESCE(account_id, 0) as account_id, COALESCE(transfer_id, 0) as transfer_id, COALESCE(transfer_account_id, 0) as transfer_account_id FROM transactions WHERE container_id = {} ORDER BY date DESC LIMIT {}", container_id, l),
            None => format!("SELECT id, amount, description, category, date, container_id, COALESCE(account_id, 0) as account_id, COALESCE(transfer_id, 0) as transfer_id, COALESCE(transfer_account_id, 0) as transfer_account_id FROM transactions WHERE container_id = {} ORDER BY date DESC", container_id),
        };

        let mut stmt = conn.prepare(&query)?;
        let transactions = stmt.query_map([], |row| {
            Ok(Transaction {
                id: row.get(0)?,
                amount: row.get(1)?,
                description: row.get(2)?,
                category: row.get(3)?,
                date: row.get(4)?,
                container_id: row.get(5)?,
                account_id: row.get(6)?,
                transfer_id: row.get(7)?,
                transfer_account_id: row.get(8)?,
            })
        })?;

        transactions.collect()
    }

    pub fn get_transactions_by_account(
        &self,
        container_id: i64,
        account_id: i64,
        limit: Option<i64>,
    ) -> Result<Vec<Transaction>> {
        let conn = self.conn.lock().unwrap();
        let base = "SELECT id, amount, description, category, date, container_id, COALESCE(account_id, 0) as account_id, COALESCE(transfer_id, 0) as transfer_id, COALESCE(transfer_account_id, 0) as transfer_account_id
                   FROM transactions
                   WHERE container_id = ?1 AND account_id = ?2
                   ORDER BY date DESC";
        let query = match limit {
            Some(l) => format!("{} LIMIT {}", base, l),
            None => base.to_string(),
        };

        let mut stmt = conn.prepare(&query)?;
        let transactions = stmt.query_map(params![container_id, account_id], |row| {
            Ok(Transaction {
                id: row.get(0)?,
                amount: row.get(1)?,
                description: row.get(2)?,
                category: row.get(3)?,
                date: row.get(4)?,
                container_id: row.get(5)?,
                account_id: row.get(6)?,
                transfer_id: row.get(7)?,
                transfer_account_id: row.get(8)?,
            })
        })?;

        transactions.collect()
    }

    pub fn update_transaction(
        &self,
        id: i64,
        amount: i64,
        description: String,
        category: String,
        account_id: i64,
    ) -> Result<Transaction> {
        let conn = self.conn.lock().unwrap();

        let transfer_id: Option<i64> = conn.query_row(
            "SELECT transfer_id FROM transactions WHERE id = ?1",
            [id],
            |row| row.get(0),
        )?;

        if transfer_id.is_some() {
            return Err(rusqlite::Error::InvalidParameterName(
                "Cannot update transfer transaction".to_string(),
            ));
        }
        
        conn.execute(
            "UPDATE transactions SET amount = ?1, description = ?2, category = ?3, account_id = ?4 WHERE id = ?5",
            params![amount, description, category, account_id, id],
        )?;

        let transaction = conn.query_row(
            "SELECT id, amount, description, category, date, container_id, COALESCE(account_id, 0) as account_id, COALESCE(transfer_id, 0) as transfer_id, COALESCE(transfer_account_id, 0) as transfer_account_id FROM transactions WHERE id = ?1",
            [id],
            |row| {
                Ok(Transaction {
                    id: row.get(0)?,
                    amount: row.get(1)?,
                    description: row.get(2)?,
                    category: row.get(3)?,
                    date: row.get(4)?,
                    container_id: row.get(5)?,
                    account_id: row.get(6)?,
                    transfer_id: row.get(7)?,
                    transfer_account_id: row.get(8)?,
                })
            },
        )?;

        Ok(transaction)
    }

    pub fn get_monthly_balance(&self, container_id: i64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let current_month = chrono::Local::now().format("%Y-%m").to_string();
        
        let balance: i64 = conn.query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM transactions WHERE container_id = ?1 AND date LIKE ?2 AND transfer_id IS NULL",
            [&container_id.to_string(), &format!("{}%", current_month)],
            |row| row.get(0),
        )?;

        Ok(balance)
    }

    pub fn get_all_time_balance(&self, container_id: i64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        
        let balance: i64 = conn.query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM transactions WHERE container_id = ?1 AND transfer_id IS NULL",
            [container_id],
            |row| row.get(0),
        )?;

        Ok(balance)
    }

    pub fn export_transactions_csv(&self, container_id: i64) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, amount, description, category, date FROM transactions WHERE container_id = ?1 ORDER BY date DESC"
        )?;
        
        let mut csv = String::from("ID,Amount,Description,Category,Date\n");
        let rows = stmt.query_map([container_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })?;

        for row in rows {
            let (id, amount, desc, cat, date) = row?;
            let dollars = (amount as f64) / 100.0;
            csv.push_str(&format!("{},{:.2},{},{},{}\n", id, dollars, desc, cat, date));
        }

        Ok(csv)
    }

    pub fn delete_transaction(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let transfer_id: i64 = conn.query_row(
            "SELECT COALESCE(transfer_id, 0) FROM transactions WHERE id = ?1",
            [id],
            |row| row.get(0),
        )?;

        if transfer_id != 0 {
            conn.execute("DELETE FROM transactions WHERE transfer_id = ?1", [transfer_id])?;
        } else {
            conn.execute("DELETE FROM transactions WHERE id = ?1", [id])?;
        }
        Ok(())
    }

    pub fn get_category_totals(&self, container_id: i64) -> Result<Vec<(String, i64)>> {
        let conn = self.conn.lock().unwrap();
        let current_month = chrono::Local::now().format("%Y-%m").to_string();
        
        let mut stmt = conn.prepare(
            "SELECT t.category, SUM(ABS(t.amount)) as total 
             FROM transactions t
             LEFT JOIN categories c ON c.name = t.category
             WHERE t.container_id = ?1 AND t.date LIKE ?2 AND t.transfer_id IS NULL
               AND COALESCE(c.category_type, 'expense') = 'expense'
             GROUP BY t.category 
             ORDER BY total DESC"
        )?;
        
        let results = stmt.query_map([&container_id.to_string(), &format!("{}%", current_month)], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        
        results.collect()
    }

    pub fn get_categories(&self) -> Result<Vec<Category>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT name, category_type, is_default FROM categories ORDER BY is_default DESC, name ASC",
        )?;
        
        let categories = stmt.query_map([], |row| {
            Ok(Category {
                name: row.get(0)?,
                category_type: row.get(1)?,
                is_default: row.get::<_, i64>(2)? == 1,
            })
        })?;
        categories.collect()
    }

    pub fn get_accounts(&self, container_id: i64) -> Result<Vec<Account>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, account_type, opening_balance, container_id, created_at
             FROM accounts
             WHERE container_id = ?1
             ORDER BY name ASC"
        )?;

        let accounts = stmt.query_map([container_id], |row| {
            Ok(Account {
                id: row.get(0)?,
                name: row.get(1)?,
                account_type: row.get(2)?,
                opening_balance: row.get(3)?,
                container_id: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;

        accounts.collect()
    }

    pub fn get_account_balances(&self, container_id: i64) -> Result<Vec<AccountBalance>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT a.id, a.name, a.account_type, a.opening_balance, a.container_id, a.created_at,
                    COALESCE(SUM(t.amount), 0) + a.opening_balance AS balance
             FROM accounts a
             LEFT JOIN transactions t ON t.account_id = a.id
             WHERE a.container_id = ?1
             GROUP BY a.id
             ORDER BY a.name ASC"
        )?;

        let accounts = stmt.query_map([container_id], |row| {
            Ok(AccountBalance {
                id: row.get(0)?,
                name: row.get(1)?,
                account_type: row.get(2)?,
                opening_balance: row.get(3)?,
                container_id: row.get(4)?,
                created_at: row.get(5)?,
                balance: row.get(6)?,
            })
        })?;

        accounts.collect()
    }

    pub fn add_account(
        &self,
        container_id: i64,
        name: String,
        account_type: String,
        opening_balance: i64,
    ) -> Result<Account> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let name = name.trim().to_string();
        let account_type = account_type.trim().to_string();

        conn.execute(
            "INSERT INTO accounts (name, account_type, opening_balance, container_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            [
                &name,
                &account_type,
                &opening_balance.to_string(),
                &container_id.to_string(),
                &now,
            ],
        )?;

        let id = conn.last_insert_rowid();

        Ok(Account {
            id,
            name,
            account_type,
            opening_balance,
            container_id,
            created_at: now,
        })
    }

    pub fn update_account(&self, id: i64, name: String, opening_balance: i64) -> Result<Account> {
        let conn = self.conn.lock().unwrap();
        let name = name.trim().to_string();

        conn.execute(
            "UPDATE accounts SET name = ?1, opening_balance = ?2 WHERE id = ?3",
            params![name, opening_balance, id],
        )?;

        let account = conn.query_row(
            "SELECT id, name, account_type, opening_balance, container_id, created_at
             FROM accounts
             WHERE id = ?1",
            [id],
            |row| {
                Ok(Account {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    account_type: row.get(2)?,
                    opening_balance: row.get(3)?,
                    container_id: row.get(4)?,
                    created_at: row.get(5)?,
                })
            },
        )?;

        Ok(account)
    }

    pub fn delete_account(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "UPDATE transactions SET account_id = NULL WHERE account_id = ?1",
            [id],
        )?;

        conn.execute("DELETE FROM accounts WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn add_category(&self, name: String, category_type: String) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO categories (name, category_type, is_default) VALUES (?1, ?2, 0)",
            [name, category_type],
        )?;
        Ok(())
    }

    pub fn delete_category(&self, name: String) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM categories WHERE name = ?1 AND is_default = 0",
            [name],
        )?;
        Ok(())
    }

    pub fn get_available_months(&self, container_id: i64) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT DISTINCT strftime('%Y-%m', date) as month 
             FROM transactions 
             WHERE container_id = ?1
             ORDER BY month DESC"
        )?;
        
        let months = stmt.query_map([container_id], |row| row.get(0))?;
        months.collect()
    }

    pub fn get_balance_for_month(&self, container_id: i64, month: String) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        
        let balance: i64 = conn.query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM transactions WHERE container_id = ?1 AND date LIKE ?2 AND transfer_id IS NULL",
            [&container_id.to_string(), &format!("{}%", month)],
            |row| row.get(0),
        )?;

        Ok(balance)
    }

    pub fn get_transactions_for_month(&self, container_id: i64, month: String, limit: Option<i64>) -> Result<Vec<Transaction>> {
        let conn = self.conn.lock().unwrap();
        let base_query = format!(
            "SELECT id, amount, description, category, date, container_id, COALESCE(account_id, 0) as account_id, COALESCE(transfer_id, 0) as transfer_id, COALESCE(transfer_account_id, 0) as transfer_account_id FROM transactions WHERE container_id = {} AND date LIKE '{}%' ORDER BY date DESC",
            container_id, month
        );
        
        let query = match limit {
            Some(l) => format!("{} LIMIT {}", base_query, l),
            None => base_query,
        };

        let mut stmt = conn.prepare(&query)?;
        let transactions = stmt.query_map([], |row| {
            Ok(Transaction {
                id: row.get(0)?,
                amount: row.get(1)?,
                description: row.get(2)?,
                category: row.get(3)?,
                date: row.get(4)?,
                container_id: row.get(5)?,
                account_id: row.get(6)?,
                transfer_id: row.get(7)?,
                transfer_account_id: row.get(8)?,
            })
        })?;

        transactions.collect()
    }

    pub fn get_category_totals_for_month(&self, container_id: i64, month: String) -> Result<Vec<(String, i64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT t.category, SUM(ABS(t.amount)) as total 
             FROM transactions t
             LEFT JOIN categories c ON c.name = t.category
             WHERE t.container_id = ?1 AND t.date LIKE ?2 AND t.transfer_id IS NULL
               AND COALESCE(c.category_type, 'expense') = 'expense'
             GROUP BY t.category 
             ORDER BY total DESC"
        )?;

        let results = stmt.query_map([&container_id.to_string(), &format!("{}%", month)], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        
        results.collect()
    }

    pub fn get_profit_and_loss_for_month(&self, container_id: i64, month: String) -> Result<ProfitLossReport> {
        let conn = self.conn.lock().unwrap();
        let (start_date, end_date) = Self::month_range(&month)?;

        let mut income_stmt = conn.prepare(
            "SELECT t.category, SUM(ABS(t.amount)) as total
             FROM transactions t
             LEFT JOIN categories c ON c.name = t.category
             WHERE t.container_id = ?1 AND t.transfer_id IS NULL
               AND t.date >= ?2 AND t.date <= ?3
               AND COALESCE(c.category_type, 'expense') = 'income'
             GROUP BY t.category
             ORDER BY total DESC",
        )?;
        let income_iter = income_stmt.query_map(
            params![container_id, &start_date, &end_date],
            |row| {
                Ok(ProfitLossLine {
                    category: row.get(0)?,
                    total: row.get(1)?,
                })
            },
        )?;
        let income: Vec<ProfitLossLine> = income_iter.collect::<Result<Vec<_>>>()?;

        let mut expense_stmt = conn.prepare(
            "SELECT t.category, SUM(ABS(t.amount)) as total
             FROM transactions t
             LEFT JOIN categories c ON c.name = t.category
             WHERE t.container_id = ?1 AND t.transfer_id IS NULL
               AND t.date >= ?2 AND t.date <= ?3
               AND COALESCE(c.category_type, 'expense') = 'expense'
             GROUP BY t.category
             ORDER BY total DESC",
        )?;
        let expense_iter = expense_stmt.query_map(
            params![container_id, &start_date, &end_date],
            |row| {
                Ok(ProfitLossLine {
                    category: row.get(0)?,
                    total: row.get(1)?,
                })
            },
        )?;
        let expense: Vec<ProfitLossLine> = expense_iter.collect::<Result<Vec<_>>>()?;

        let total_income: i64 = income.iter().map(|line| line.total).sum();
        let total_expense: i64 = expense.iter().map(|line| line.total).sum();
        let net_income = total_income - total_expense;

        Ok(ProfitLossReport {
            start_date,
            end_date,
            income,
            expense,
            total_income,
            total_expense,
            net_income,
        })
    }

    pub fn get_balance_sheet_for_month(&self, container_id: i64, month: String) -> Result<BalanceSheetReport> {
        let conn = self.conn.lock().unwrap();
        let (_start_date, end_date) = Self::month_range(&month)?;

        let mut stmt = conn.prepare(
            "SELECT a.id, a.name, a.account_type, a.opening_balance, a.container_id, a.created_at,
                    COALESCE(SUM(t.amount), 0) + a.opening_balance AS balance
             FROM accounts a
             LEFT JOIN transactions t ON t.account_id = a.id AND t.date <= ?2
             WHERE a.container_id = ?1
             GROUP BY a.id
             ORDER BY a.name ASC",
        )?;

        let accounts_iter = stmt.query_map(params![container_id, &end_date], |row| {
            Ok(AccountBalance {
                id: row.get(0)?,
                name: row.get(1)?,
                account_type: row.get(2)?,
                opening_balance: row.get(3)?,
                container_id: row.get(4)?,
                created_at: row.get(5)?,
                balance: row.get(6)?,
            })
        })?;

        let mut assets = Vec::new();
        let mut liabilities = Vec::new();
        let mut equity = Vec::new();

        for account in accounts_iter {
            let account = account?;
            match account.account_type.as_str() {
                "asset" | "contra_asset" => assets.push(account),
                "liability" => liabilities.push(account),
                _ => equity.push(account),
            }
        }

        let total_assets: i64 = assets.iter().map(|a| a.balance).sum();
        let total_liabilities: i64 = liabilities.iter().map(|a| a.balance).sum();
        let total_equity: i64 = equity.iter().map(|a| a.balance).sum();

        Ok(BalanceSheetReport {
            as_of: end_date,
            assets,
            liabilities,
            equity,
            total_assets,
            total_liabilities,
            total_equity,
        })
    }

    pub fn get_containers(&self) -> Result<Vec<Container>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, created_at, is_default FROM containers ORDER BY is_default DESC, created_at ASC")?;
        
        let containers = stmt.query_map([], |row| {
            Ok(Container {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
                is_default: row.get::<_, i64>(3)? == 1,
            })
        })?;
        
        containers.collect()
    }

    pub fn add_container(&self, name: String) -> Result<Container> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        conn.execute(
            "INSERT INTO containers (name, created_at, is_default) VALUES (?1, ?2, 0)",
            [&name, &now],
        )?;

        let id = conn.last_insert_rowid();
        
        Ok(Container {
            id,
            name,
            created_at: now,
            is_default: false,
        })
    }

    pub fn delete_container(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        let is_default: i64 = conn.query_row(
            "SELECT is_default FROM containers WHERE id = ?1",
            [id],
            |row| row.get(0),
        )?;
        
        if is_default == 1 {
            return Err(rusqlite::Error::InvalidParameterName("Cannot delete default container".to_string()));
        }
        
        conn.execute("DELETE FROM containers WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn update_container(&self, id: i64, name: String) -> Result<Container> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "UPDATE containers SET name = ?1 WHERE id = ?2",
            [&name, &id.to_string()],
        )?;

        let container = conn.query_row(
            "SELECT id, name, created_at, is_default FROM containers WHERE id = ?1",
            [id],
            |row| {
                Ok(Container {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                    is_default: row.get::<_, i64>(3)? == 1,
                })
            },
        )?;

        Ok(container)
    }

    fn month_range(month: &str) -> Result<(String, String)> {
        let parts: Vec<&str> = month.split('-').collect();
        if parts.len() != 2 {
            return Err(rusqlite::Error::InvalidParameterName(
                "Invalid month format".to_string(),
            ));
        }

        let year: i32 = parts[0].parse().map_err(|_| {
            rusqlite::Error::InvalidParameterName("Invalid year".to_string())
        })?;
        let month_num: u32 = parts[1].parse().map_err(|_| {
            rusqlite::Error::InvalidParameterName("Invalid month".to_string())
        })?;

        let start = chrono::NaiveDate::from_ymd_opt(year, month_num, 1).ok_or_else(|| {
            rusqlite::Error::InvalidParameterName("Invalid month".to_string())
        })?;

        let (next_year, next_month) = if month_num == 12 {
            (year + 1, 1)
        } else {
            (year, month_num + 1)
        };

        let end = chrono::NaiveDate::from_ymd_opt(next_year, next_month, 1)
            .and_then(|d| d.pred_opt())
            .ok_or_else(|| rusqlite::Error::InvalidParameterName("Invalid month".to_string()))?;

        let start_date = format!("{} 00:00:00", start.format("%Y-%m-%d"));
        let end_date = format!("{} 23:59:59", end.format("%Y-%m-%d"));

        Ok((start_date, end_date))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportResult {
    pub success_count: usize,
    pub error_count: usize,
    pub errors: Vec<String>,
}

impl Database {
    pub fn import_transactions_from_csv(
        &self,
        csv_content: String,
        container_id: i64,
        amount_column: usize,
        description_column: usize,
        category_column: usize,
        date_column: usize,
        skip_header: bool,
    ) -> Result<ImportResult> {
        let mut reader = ReaderBuilder::new()
            .has_headers(skip_header)
            .from_reader(csv_content.as_bytes());

        let mut success_count = 0;
        let mut error_count = 0;
        let mut errors = Vec::new();

        for (index, result) in reader.records().enumerate() {
            let row_num = if skip_header { index + 2 } else { index + 1 };
            
            match result {
                Ok(record) => {
                    let amount_str = record.get(amount_column).unwrap_or("").trim();
                    let description = record.get(description_column).unwrap_or("Imported").trim().to_string();
                    let category = record.get(category_column).unwrap_or("Other").trim().to_string();
                    let date_str = record.get(date_column).unwrap_or("").trim();

                    let amount_cents = match Self::parse_amount(amount_str) {
                        Ok(amt) => amt,
                        Err(e) => {
                            errors.push(format!("Row {}: Invalid amount '{}' - {}", row_num, amount_str, e));
                            error_count += 1;
                            continue;
                        }
                    };

                    let parsed_date = match Self::parse_date(date_str) {
                        Ok(date) => date,
                        Err(e) => {
                            errors.push(format!("Row {}: Invalid date '{}' - {}", row_num, date_str, e));
                            error_count += 1;
                            continue;
                        }
                    };

                    match self.insert_imported_transaction(
                        container_id,
                        amount_cents,
                        description,
                        category,
                        parsed_date,
                    ) {
                        Ok(_) => success_count += 1,
                        Err(e) => {
                            errors.push(format!("Row {}: Failed to insert - {}", row_num, e));
                            error_count += 1;
                        }
                    }
                }
                Err(e) => {
                    errors.push(format!("Row {}: Failed to parse CSV - {}", row_num, e));
                    error_count += 1;
                }
            }
        }

        Ok(ImportResult {
            success_count,
            error_count,
            errors,
        })
    }

    fn parse_amount(amount_str: &str) -> Result<i64, String> {
        let cleaned = amount_str
            .replace("$", "")
            .replace("€", "")
            .replace("£", "")
            .replace(",", "")
            .trim()
            .to_string();

        match cleaned.parse::<f64>() {
            Ok(amount) => Ok((amount * 100.0).round() as i64),
            Err(_) => Err(format!("Cannot parse as number")),
        }
    }

    fn parse_date(date_str: &str) -> Result<String, String> {
        let formats = vec![
            "%Y-%m-%d",
            "%m/%d/%Y",
            "%d/%m/%Y",
            "%Y/%m/%d",
            "%m-%d-%Y",
            "%d-%m-%Y",
            "%Y-%m-%d %H:%M:%S",
            "%m/%d/%Y %H:%M",
        ];

        for format in formats {
            if let Ok(parsed) = chrono::NaiveDateTime::parse_from_str(&format!("{} 00:00:00", date_str), "%Y-%m-%d %H:%M:%S") {
                return Ok(parsed.format("%Y-%m-%d %H:%M:%S").to_string());
            }
            if let Ok(parsed) = chrono::NaiveDateTime::parse_from_str(date_str, format) {
                return Ok(parsed.format("%Y-%m-%d %H:%M:%S").to_string());
            }
            if let Ok(parsed) = chrono::NaiveDate::parse_from_str(date_str, format) {
                let datetime = parsed.and_hms_opt(0, 0, 0).unwrap();
                return Ok(datetime.format("%Y-%m-%d %H:%M:%S").to_string());
            }
        }

        Err("Unsupported date format".to_string())
    }

    fn insert_imported_transaction(
        &self,
        container_id: i64,
        amount: i64,
        description: String,
        category: String,
        date: String,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "INSERT INTO transactions (amount, description, category, date, container_id) VALUES (?1, ?2, ?3, ?4, ?5)",
            [
                &amount.to_string(),
                &description,
                &category,
                &date,
                &container_id.to_string(),
            ],
        )?;

        Ok(())
    }
}
