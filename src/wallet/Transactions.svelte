
<script>
    import { invoke } from "@tauri-apps/api";
    import Spinner from "../utils/Spinner.svelte";

    export let coin_index;
    export let account_index;

    let TOKEN_NAME = {
        0: 'BTC',
        1: 'TBTC'
    };
    let TRANSACTION_COLOR = {
        'Incoming': 'text-green-800',
        'Outgoing': 'text-red-800',
        'Internal': 'black'
    };
    let SATOSHI = 100000000;

    function getSimpleTransactions() {
        return invoke('get_simple_transactions', {coinTypeIndex: coin_index, accountIndex: account_index});
    }

</script>

{#await getSimpleTransactions()}
    <div class="pl-4 pt-8">
        <Spinner></Spinner>
    </div>
{:then simpleTransactions}
    <div class="pt-4 pl-2">
        <div class="table-wrp block max-h-96 overflow-x-auto">
            <table class="table w-full">
                <thead class="border-b sticky top-0">
                <tr>
                    <th>Type</th>
                    <th>Amount [{TOKEN_NAME[coin_index]}]</th>
                    <th>Fee [{TOKEN_NAME[coin_index]}]</th>
                    <th>Status</th>
                </tr>
                </thead>
                <tbody>
                {#each simpleTransactions as tx}
                    <tr>
                        <td><span class={TRANSACTION_COLOR[tx.transaction_type]}>{tx.transaction_type}</span></td>
                        <td>{tx.value / SATOSHI}</td>
                        <td>{tx.fee / SATOSHI}</td>
                        <td>
                            <div class={tx.confirmed ? 'badge badge-success gap-1' : 'badge badge-warning gap-1'}>
                                {tx.confirmed ? 'confirmed' : 'unconfirmed'}
                            </div>
                        </td>
                    </tr>
                {/each}
                </tbody>
            </table>
        </div>
    </div>
{:catch error}
<div class="pt-8">
    <div class="alert alert-error shadow-lg">
        <div>
            <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current flex-shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
            <span>Unable to load transactions.</span>
        </div>
    </div>
</div>
{/await}


<style>
    .table-wrp  {
        overflow-y: auto;
        display:block;
    }
    thead{
        position:sticky;
        top:0
    }
</style>