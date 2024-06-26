
<a name="0x4_genesis"></a>

# Module `0x4::genesis`



-  [Struct `BitcoinGenesisContext`](#0x4_genesis_BitcoinGenesisContext)
-  [Constants](#@Constants_0)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x2::signer</a>;
<b>use</b> <a href="">0x2::tx_context</a>;
<b>use</b> <a href="brc20.md#0x4_brc20">0x4::brc20</a>;
<b>use</b> <a href="data_import_config.md#0x4_data_import_config">0x4::data_import_config</a>;
<b>use</b> <a href="light_client.md#0x4_light_client">0x4::light_client</a>;
<b>use</b> <a href="network.md#0x4_network">0x4::network</a>;
<b>use</b> <a href="ord.md#0x4_ord">0x4::ord</a>;
<b>use</b> <a href="utxo.md#0x4_utxo">0x4::utxo</a>;
</code></pre>



<a name="0x4_genesis_BitcoinGenesisContext"></a>

## Struct `BitcoinGenesisContext`

BitcoinGenesisContext is a genesis init config in the TxContext.


<pre><code><b>struct</b> <a href="genesis.md#0x4_genesis_BitcoinGenesisContext">BitcoinGenesisContext</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x4_genesis_ErrorGenesisInit"></a>



<pre><code><b>const</b> <a href="genesis.md#0x4_genesis_ErrorGenesisInit">ErrorGenesisInit</a>: u64 = 1;
</code></pre>
