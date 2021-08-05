use crate::{Iroha, Transaction};

pub struct Client<'a>(&'a mut Iroha);

impl<'a> Client<'a> {
    pub fn new(iroha: &'a mut Iroha) -> Self {
        Client(iroha)
    }

    pub fn submit_transaction(&mut self, transaction: Transaction) {
        transaction.execute(self.0)
    }
}
