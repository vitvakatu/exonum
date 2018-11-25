// Copyright 2018 The Exonum Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Cryptocurrency transactions.

// Workaround for `failure` see https://github.com/rust-lang-nursery/failure/issues/223 and
// ECR-1771 for the details.
#![allow(bare_trait_objects)]

use exonum::{
    blockchain::{ExecutionError, ExecutionResult, Transaction},
    crypto::{CryptoHash, Hash, PublicKey},
    messages::Message,
    storage::Fork,
};

use schema::Schema;
use CRYPTOCURRENCY_SERVICE_ID;

/// Error codes emitted by wallet transactions during execution.
#[derive(Debug, Fail)]
#[repr(u8)]
pub enum Error {
    /// Wallet already exists.
    ///
    /// Can be emitted by `CreateWallet`
    #[fail(display = "Wallet already exists")]
    WalletAlreadyExists = 0,

    /// Sender doesn't exist.
    ///
    /// Can be emitted by `Transfer`.
    #[fail(display = "Sender doesn't exist")]
    SenderNotFound = 1,

    /// Receiver doesn't exist.
    ///
    /// Can be emitted by `Transfer` or `Issue`.
    #[fail(display = "Receiver doesn't exist")]
    ReceiverNotFound = 2,

    /// Insufficient currency amount.
    ///
    /// Can be emitted by `Transfer`.
    #[fail(display = "Insufficient currency amount")]
    InsufficientCurrencyAmount = 3,

    /// Error for used Hash
    #[fail(display = "Used hash")]
    UsedHash = 4,
}

impl From<Error> for ExecutionError {
    fn from(value: Error) -> ExecutionError {
        let description = format!("{}", value);
        ExecutionError::with_description(value as u8, description)
    }
}

transactions! {
    /// Transaction group.
    pub WalletTransactions {
        const SERVICE_ID = CRYPTOCURRENCY_SERVICE_ID;

        /// Transfer `amount` of the currency from one wallet to another.
        struct Transfer {
            tx_hash1: &Hash,
            tx_hash2: &Hash,
            /// `PublicKey` of sender's wallet.
            from1:    &PublicKey,
            from2:    &PublicKey,
            /// `PublicKey` of receiver's wallet.
            to1:      &PublicKey,
            to2:      &PublicKey,
            change1: &PublicKey,
            change2: &PublicKey,
            /// Amount of currency to transfer.
            amount1:  u64,
            amount2:  u64,
            loss1: u64,
            loss2: u64,
            /// Auxiliary number to guarantee [non-idempotence][idempotence] of transactions.
            ///
            /// [idempotence]: https://en.wikipedia.org/wiki/Idempotence
            seed:    u64,
        }

        /// Issue `amount` of the currency to the `wallet`.
        struct Issue {
            /// `PublicKey` of the wallet.
            pub_key:  &PublicKey,
            /// Issued amount of currency.
            amount:  u64,
            /// Auxiliary number to guarantee [non-idempotence][idempotence] of transactions.
            ///
            /// [idempotence]: https://en.wikipedia.org/wiki/Idempotence
            seed:    u64,
        }

        /// Create wallet with the given `name`.
        struct CreateWallet {
            /// `PublicKey` of the new wallet.
            pub_key: &PublicKey,
            /// Name of the new wallet.
            name:    &str,
        }

        struct Transfer_One {
            tx_hash: &Hash,
            /// `PublicKey` of sender's wallet.
            from:    &PublicKey,
            /// `PublicKey` of receiver's wallet.
            to:      &PublicKey,
            change: &PublicKey,
            /// Amount of currency to transfer.
            amount:  u64,
            /// amount loss
            loss: u64,
            /// Auxiliary number to guarantee [non-idempotence][idempotence] of transactions.
            ///
            /// [idempotence]: https://en.wikipedia.org/wiki/Idempotence
            seed:    u64,
        }
    }
}


impl Transaction for Transfer_One {
    fn verify(&self) -> bool {
        (self.from() != self.to()) && self.verify_signature(self.from())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let mut schema = Schema::new(fork);

        let from = self.from();
        let change = from;
        let to = self.to();
        let hash = self.hash();
        let amount = self.amount();
        let loss = self.loss();
        let tx_hash = self.tx_hash();

        let sender = schema.wallet(from).ok_or(Error::SenderNotFound)?;

        let mut used = sender.used().to_vec();

        for i in &used {
            if (i == tx_hash) {
                println!(
                    "ERROR: This hash was used before"
                );
                Err(Error::UsedHash)?
            }
        }

        used.push(*tx_hash);

        let receiver = schema.wallet(to).ok_or(Error::ReceiverNotFound)?;
        let new = receiver.used().to_vec();
        if sender.balance() < amount {
            Err(Error::InsufficientCurrencyAmount)?
        }

        schema.decrease_wallet_balance(sender, amount, &hash, &used);
        schema.increase_wallet_balance(receiver, amount, &hash, &new);

        Ok(())
    }
}


impl Transaction for Transfer {
    fn verify(&self) -> bool {

        (self.from1() != self.to1()) && self.verify_signature(self.from1())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let mut schema = Schema::new(fork);

        println!(
            "start execute"
        );

        let from1 = self.from1();
        let from2 = self.from2();
        let to1 = self.to1();
        let to2 = self.to2();

        let amount1 = self.amount1();
        let amount2 = self.amount2();
        let loss1 = self.loss1();
        let loss2 = self.loss2();
        let tx_hash1 = self.tx_hash1();
        let tx_hash2 = self.tx_hash2();

        let hash = self.hash();

        let sender1 = schema.wallet(from1).ok_or(Error::SenderNotFound)?;
        let sender2 = schema.wallet(from2).ok_or(Error::SenderNotFound)?;

        let mut used1 = sender1.used().to_vec();

        for i in &used1 {
            if (i == tx_hash1) {
                println!(
                    "ERROR: This hash was used before"
                );
                Err(Error::UsedHash)?
            }
        }

        used1.push(*tx_hash1);

        let mut used2 = sender2.used().to_vec();

        for i in &used2 {
            if (i == tx_hash2) {
                println!(
                    "ERROR: This hash was used before"
                );
                Err(Error::UsedHash)?
            }
        }

        used1.push(*tx_hash2);

        let receiver1 = schema.wallet(to1).ok_or(Error::ReceiverNotFound)?;
        let receiver2 = schema.wallet(to2).ok_or(Error::ReceiverNotFound)?;
        let new1 = receiver1.used().to_vec();
        let new2 = receiver2.used().to_vec();
        if sender1.balance() < amount1 {
            Err(Error::InsufficientCurrencyAmount)?
        }

        schema.decrease_wallet_balance(sender1, amount1, &hash, &used1);
        schema.decrease_wallet_balance(sender2, amount2, &hash, &used2);
        schema.increase_wallet_balance(receiver1, amount1, &hash, &new1);
        schema.increase_wallet_balance(receiver2, amount2, &hash, &new2);

        println!(
            "ended execute"
        );

        Ok(())
    }
}

impl Transaction for Issue {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let mut schema = Schema::new(fork);
        let pub_key = self.pub_key();
        let hash = self.hash();

        if let Some(wallet) = schema.wallet(pub_key) {
            let amount = self.amount();
            let new = wallet.used().to_vec();
            schema.increase_wallet_balance(wallet, amount, &hash, &new);
            Ok(())
        } else {
            Err(Error::ReceiverNotFound)?
        }
    }
}

impl Transaction for CreateWallet {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let mut schema = Schema::new(fork);
        let pub_key = self.pub_key();
        let hash = self.hash();

        if schema.wallet(pub_key).is_none() {
            let name = self.name();
            let using = Vec::new();
            schema.create_wallet(pub_key, name, &hash, &using);
            Ok(())
        } else {
            Err(Error::WalletAlreadyExists)?
        }
    }
}
