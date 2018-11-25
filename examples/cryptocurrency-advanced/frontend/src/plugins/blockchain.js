import * as Exonum from 'exonum-client'
import axios from 'axios'

const TRANSACTION_URL = '/api/services/cryptocurrency/v1/wallets/transaction'
const TRANSACTION_EXPLORER_URL = '/api/explorer/v1/transactions?hash='
const PER_PAGE = 10
const PROTOCOL_VERSION = 0
const SERVICE_ID = 128
const TX_TRANSFER_ID = 0
const TX_ISSUE_ID = 1
const TX_WALLET_ID = 2
const TX_TRANSFER_O_ID = 3

const used = Exonum.newArray({
  type: Exonum.Hash
})

const TableKey = Exonum.newType({
  fields: [
    { name: 'service_id', type: Exonum.Uint16 },
    { name: 'table_index', type: Exonum.Uint16 }
  ]
})
const Wallet = Exonum.newType({
  fields: [
    { name: 'pub_key', type: Exonum.PublicKey },
    { name: 'name', type: Exonum.String },
    { name: 'balance', type: Exonum.Uint64 },
    { name: 'history_len', type: Exonum.Uint64 },
    { name: 'history_hash', type: Exonum.Hash },
    { name: 'used', type: used }
  ]
})
const TransactionMetaData = Exonum.newType({
  fields: [
    { name: 'tx_hash', type: Exonum.Hash },
    { name: 'execution_status', type: Exonum.Bool }
  ]
})

function TransferTransaction () {
  return Exonum.newMessage({
    protocol_version: PROTOCOL_VERSION,
    service_id: SERVICE_ID,
    message_id: TX_TRANSFER_ID,
    fields: [
      { name: 'tx_hash1', type: Exonum.Hash },
      { name: 'tx_hash2', type: Exonum.Hash },
      { name: 'from1', type: Exonum.PublicKey },
      { name: 'from2', type: Exonum.PublicKey },
      { name: 'to1', type: Exonum.PublicKey },
      { name: 'to2', type: Exonum.PublicKey },
      { name: 'change1', type: Exonum.PublicKey },
      { name: 'change2', type: Exonum.PublicKey },
      { name: 'amount1', type: Exonum.Uint64 },
      { name: 'amount2', type: Exonum.Uint64 },
      { name: 'loss1', type: Exonum.Uint64 },
      { name: 'loss2', type: Exonum.Uint64 },
      { name: 'seed', type: Exonum.Uint64 }
    ]
  })
}

function TransferTransaction_One () {
  return Exonum.newMessage({
    protocol_version: PROTOCOL_VERSION,
    service_id: SERVICE_ID,
    message_id: TX_TRANSFER_O_ID,
    fields: [
      { name: 'tx_hash', type: Exonum.Hash },
      { name: 'from', type: Exonum.PublicKey },
      { name: 'to', type: Exonum.PublicKey },
      { name: 'change', type: Exonum.PublicKey },
      { name: 'amount', type: Exonum.Uint64 },
      { name: 'loss', type: Exonum.Uint64 },
      { name: 'seed', type: Exonum.Uint64 }
    ]
  })
}


function IssueTransaction () {
  return Exonum.newMessage({
    protocol_version: PROTOCOL_VERSION,
    service_id: SERVICE_ID,
    message_id: TX_ISSUE_ID,
    fields: [
      { name: 'pub_key', type: Exonum.PublicKey },
      { name: 'amount', type: Exonum.Uint64 },
      { name: 'seed', type: Exonum.Uint64 }
    ]
  })
}

function CreateTransaction () {
  return Exonum.newMessage({
    protocol_version: PROTOCOL_VERSION,
    service_id: SERVICE_ID,
    message_id: TX_WALLET_ID,
    fields: [
      { name: 'pub_key', type: Exonum.PublicKey },
      { name: 'name', type: Exonum.String }
    ]
  })
}

function getTransaction(id) {
  switch (id) {
    case TX_TRANSFER_ID:
      return new TransferTransaction()
    case TX_TRANSFER_O_ID:
      return new TransferTransaction_One()
    case TX_ISSUE_ID:
      return new IssueTransaction()
    case TX_WALLET_ID:
      return new CreateTransaction()
    default:
      throw new Error('Unknown transaction ID has been passed')
  }
}

function getOwner(transaction) {
  switch (transaction.message_id) {
    case TX_TRANSFER_ID:
      return transaction.body.from1
    case TX_TRANSFER_O_ID:
      return transaction.body.from
    case TX_ISSUE_ID:
      return transaction.body.pub_key
    case TX_WALLET_ID:
      return transaction.body.pub_key
    default:
      throw new Error('Unknown transaction ID has been passed')
  }
}

module.exports = {
  install(Vue) {
    Vue.prototype.$blockchain = {
      generateKeyPair() {
        return Exonum.keyPair()
      },

      generateSeed() {
        return Exonum.randomUint64()
      },

      createWallet(keyPair, name) {
        // Describe transaction
        const transaction = new CreateTransaction()

        // Transaction data
        const data = {
          pub_key: keyPair.publicKey,
          name: name
        }

        // Sign transaction
        const signature = transaction.sign(keyPair.secretKey, data)

        // Send transaction into blockchain
        return transaction.send(TRANSACTION_URL, TRANSACTION_EXPLORER_URL, data, signature)
      },

      addFunds(keyPair, amountToAdd, seed) {
        // Describe transaction
        const transaction = new IssueTransaction()

        // Transaction data
        const data = {
          pub_key: keyPair.publicKey,
          amount: amountToAdd.toString(),
          seed: seed
        }

        // Sign transaction
        const signature = transaction.sign(keyPair.secretKey, data)

        // Send transaction into blockchain
        return transaction.send(TRANSACTION_URL, TRANSACTION_EXPLORER_URL, data, signature)
      },

      transfer_one(keyPair, tx_hash, from, receiver, amountToTransfer, seed, trx) {
        // Describe transaction
        const transaction = new TransferTransaction_One()

        const tx = trx.body

        var tmp_loss


        if (trx.message_id == 1) {
            if (tx.pub_key == from) {
                tmp_loss = (parseInt(tx.amount, 10) - amountToTransfer).toString()
            } else {
                throw new Error('Wrong Sender')
            }
        }

        if (trx.message_id == 3) {
            if (tx.change == from) {
                tmp_loss = (parseInt(tx.loss, 10) - amountToTransfer).toString()
            } else {
                if (tx.to == from) {
                    tmp_loss = (parseInt(tx.amount, 10) - amountToTransfer).toString()
                } else {
                    throw new Error('Wrong Sender')
                }
            }
        }

        /*console.log(tx)
        console.log(typeof(parseInt(tx.amount, 10) - 1))
        console.log(1)*/

        // Transaction data
        const data = {
          tx_hash: tx_hash,
          from: keyPair.publicKey,
          to: receiver,
          change: from,
          amount: amountToTransfer,
          loss: tmp_loss,
          seed: seed
        }

        // Sign transaction
        const signature = transaction.sign(keyPair.secretKey, data)
        transaction.signature = signature

        const hash = transaction.hash(data)

        // Send transaction into blockchain

        //return hash
        return { tx_hash:hash,
                 send: transaction.send(TRANSACTION_URL, TRANSACTION_EXPLORER_URL, data, signature) }
      },

      transfer(keyPair, tx_hash1, from1, receiver1, amountToTransfer1, tx_hash2, from2, receiver2, amountToTransfer2, seed, trx1, trx2) {
        // Describe transaction
        const transaction = new TransferTransaction()

        console.log("transfer js started")

        const tx1 = trx1.body
        const tx2 = trx2.body

        var tmp_loss1
        var tmp_loss2


        if (trx1.message_id == 1) {
            if (tx1.pub_key == from1) {
                tmp_loss1 = (parseInt(tx1.amount, 10) - amountToTransfer1).toString()
            } else {
                throw new Error('Wrong Sender')
            }
        }

        if (trx2.message_id == 1) {
            if (tx2.pub_key == from2) {
                tmp_loss2 = (parseInt(tx2.amount, 10) - amountToTransfer2).toString()
            } else {
                throw new Error('Wrong Sender')
            }
        }


        if (trx1.message_id == 0) {
            if (tx1.change1 == from1) {
                tmp_loss1 = (parseInt(tx1.loss1, 10) - amountToTransfer1).toString()
            } else {
                if (tx1.change2 == from1) {
                    tmp_loss1 = (parseInt(tx1.loss2, 10) - amountToTransfer1).toString()
                }
                else {
                    if (tx1.to1 == from1) {
                        tmp_loss1 = (parseInt(tx1.amount1, 10) - amountToTransfer1).toString()
                    } else {
                        if (tx1.to2 == from1) {
                            tmp_loss1 = (parseInt(tx1.amount2, 10) - amountToTransfer1).toString()
                        } else {
                            throw new Error('Wrong Sender')
                        }
                    }
                }
            }
        }

        if (trx2.message_id == 0) {
            if (tx2.change1 == from2) {
                tmp_loss2 = (parseInt(tx2.loss1, 10) - amountToTransfer2).toString()
            } else {
                if (tx2.change2 == from2) {
                    tmp_loss2 = (parseInt(tx2.loss2, 10) - amountToTransfer2).toString()
                }
                else {
                    if (tx2.to1 == from2) {
                        tmp_loss2 = (parseInt(tx2.amount1, 10) - amountToTransfer2).toString()
                    } else {
                        if (tx2.to2 == from2) {
                            tmp_loss2 = (parseInt(tx2.amount2, 10) - amountToTransfer2).toString()
                        } else {
                            throw new Error('Wrong Sender')
                        }
                    }
                }
            }
        }


        if (trx1.message_id == 3) {
            if (tx1.change == from1) {
                tmp_loss1 = (parseInt(tx1.loss, 10) - amountToTransfer1).toString()
            } else {
                if (tx1.to == from1) {
                    tmp_loss1 = (parseInt(tx1.amount, 10) - amountToTransfer1).toString()
                } else {
                    throw new Error('Wrong Sender')
                }
            }
        }

        if (trx2.message_id == 3) {
            if (tx2.change == from2) {
                tmp_loss2 = (parseInt(tx2.loss, 10) - amountToTransfer2).toString()
            } else {
                if (tx2.to == from2) {
                    tmp_loss2 = (parseInt(tx2.amount, 10) - amountToTransfer2).toString()
                } else {
                    throw new Error('Wrong Sender')
                }
            }
        }


        /*console.log(tx)
        console.log(typeof(parseInt(tx.amount, 10) - 1))
        console.log(1)*/

        // Transaction data
        const data = {
          tx_hash1: tx_hash1,
          tx_hash2: tx_hash2,
          from1: from1,
          from2: from2,
          to1: receiver1,
          to2: receiver2,
          change1: from1,
          change2: from2,
          amount1: amountToTransfer1,
          amount2: amountToTransfer2,
          loss1: tmp_loss1,
          loss2: tmp_loss2,
          seed: seed
        }

        // Sign transaction
        const signature = transaction.sign(keyPair.secretKey, data)
        transaction.signature = signature

        const hash = transaction.hash(data)

        // Send transaction into blockchain

        //return hash
        console.log("trx sending")
        return { tx_hash:hash,
                 send: transaction.send(TRANSACTION_URL, TRANSACTION_EXPLORER_URL, data, signature) }
      },

      getTransaction_status(hash) {
        return axios.get(`/api/explorer/v1/transactions?hash=${hash}`).then(response => response.status)
    },

      getWallet(publicKey) {
        return axios.get('/api/services/configuration/v1/configs/actual').then(response => {
          // actual list of public keys of validators
          const validators = response.data.config.validator_keys.map(validator => {
            return validator.consensus_key
          })

          return axios.get(`/api/services/cryptocurrency/v1/wallets/info?pub_key=${publicKey}`)
            .then(response => response.data)
            .then(data => {
              if (!Exonum.verifyBlock(data.block_proof, validators)) {
                throw new Error('Block can not be verified')
              }

              // find root hash of table with wallets in the tree of all tables
              const tableKey = TableKey.hash({
                service_id: SERVICE_ID,
                table_index: 0
              })
              const tableProof = new Exonum.MapProof(data.wallet_proof.to_table, Exonum.Hash, Exonum.Hash)
              if (tableProof.merkleRoot !== data.block_proof.block.state_hash) {
                throw new Error('Wallets table proof is corrupted')
              }
              const walletsHash = tableProof.entries.get(tableKey)
              if (typeof walletsHash === 'undefined') {
                throw new Error('Wallets table not found')
              }

              // find wallet in the tree of all wallets
              const walletProof = new Exonum.MapProof(data.wallet_proof.to_wallet, Exonum.PublicKey, Wallet)
              if (walletProof.merkleRoot !== walletsHash) {
                throw new Error('Wallet proof is corrupted')
              }
              const wallet = walletProof.entries.get(publicKey)
              if (typeof wallet === 'undefined') {
                throw new Error('Wallet not found')
              }

              // get transactions
              const transactionsMetaData = Exonum.merkleProof(
                wallet.history_hash,
                wallet.history_len,
                data.wallet_history.proof,
                [0, wallet.history_len],
                TransactionMetaData
              )

              if (data.wallet_history.transactions.length !== transactionsMetaData.length) {
                // number of transactions in wallet history is not equal
                // to number of transactions in array with transactions meta data
                throw new Error('Transactions can not be verified')
              }

              // validate each transaction
              let transactions = []
              let last_hash = ""
              for (let i in data.wallet_history.transactions) {
                let transaction = data.wallet_history.transactions[i]

                // get transaction definition
                let Transaction = getTransaction(transaction.message_id)

                // get transaction owner
                const owner = getOwner(transaction)

                // add a signature to the transaction definition
                Transaction.signature = transaction.signature

                // validate transaction hash
                if (Transaction.hash(transaction.body) !== transactionsMetaData[i]) {
                  throw new Error('Invalid transaction hash has been found')
                }

                // validate transaction signature
                if (!Transaction.verifySignature(transaction.signature, owner, transaction.body)) {
                  throw new Error('Invalid transaction signature has been found')
                }

                // add transaction to the resulting array
                transactions.push(Object.assign({
                  hash: transactionsMetaData[i]
                }, transaction))
              }

              return {
                block: data.block_proof.block,
                wallet: wallet,
                transactions: transactions
              }
            })
        })
      },

      getBlocks(latest) {
        const suffix = !isNaN(latest) ? '&latest=' + latest : ''
        return axios.get(`/api/explorer/v1/blocks?count=${PER_PAGE}${suffix}`).then(response => response.data)
      },

      getBlock(height) {
        return axios.get(`/api/explorer/v1/block?height=${height}`).then(response => response.data)
      },

      getTransaction(hash) {
        return axios.get(`/api/explorer/v1/transactions?hash=${hash}`).then(response => response.data)
      }
    }
  }
}
