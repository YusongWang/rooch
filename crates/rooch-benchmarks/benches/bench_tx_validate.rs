// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use criterion::{criterion_group, criterion_main, Criterion};
use rooch_benchmarks::helper::profiled;
use rooch_benchmarks::tx::TxType::{Blog, Empty, Transfer};
use rooch_benchmarks::tx::{create_l2_tx, create_publish_transaction, TX_TYPE};
use rooch_framework_tests::binding_test;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_key::keystore::memory_keystore::InMemKeystore;
use rooch_test_transaction_builder::TestTransactionBuilder;
use std::time::Duration;

pub fn tx_validate_benchmark(c: &mut Criterion) {
    let mut binding_test = binding_test::RustBindingTest::new().unwrap();
    let keystore = InMemKeystore::new_insecure_for_tests(10);

    let default_account = keystore.addresses()[0];
    let mut test_transaction_builder = TestTransactionBuilder::new(default_account.into());

    let mut tx_cnt = 1000;
    if *TX_TYPE == Blog {
        let tx = create_publish_transaction(&test_transaction_builder, &keystore).unwrap();
        binding_test.execute(tx).unwrap();
    }
    if *TX_TYPE == Transfer {
        tx_cnt = 1000;
    }
    if *TX_TYPE == Empty {
        tx_cnt = 1000;
    }
    let transactions: Vec<_> = (0..tx_cnt)
        .map(|n| create_l2_tx(&mut test_transaction_builder, &keystore, n).unwrap())
        .collect();
    let mut transactions_iter = transactions.into_iter().cycle();

    c.bench_function("validate_tx", |b| {
        b.iter(|| {
            let tx = transactions_iter.next().unwrap();
            binding_test.executor.validate_l2_tx(tx.clone()).unwrap()
        });
    });
}

criterion_group! {
    name = tx_validate_bench;
    config = profiled(None).measurement_time(Duration::from_millis(500));
    targets = tx_validate_benchmark
}

criterion_main!(tx_validate_bench);
