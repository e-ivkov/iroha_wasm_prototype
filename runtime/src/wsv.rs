use std::collections::HashMap;

use data_model::{Account, AccountName, Instruction, Query, QueryResult};

#[derive(Debug)]
pub struct WSV {
    accounts: HashMap<AccountName, Account>,
}

impl WSV {
    pub fn execute_instruction(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Mint(amount, account) => {
                self.accounts.get_mut(&account).unwrap().balance += amount
            }
            Instruction::Burn(amount, account) => {
                self.accounts.get_mut(&account).unwrap().balance -= amount
            }
        }
    }

    pub fn execute_query(&self, query: Query) -> QueryResult {
        match query {
            Query::GetBalance(account) => {
                QueryResult::Balance(self.accounts.get(&account).unwrap().balance)
            }
        }
    }
}
