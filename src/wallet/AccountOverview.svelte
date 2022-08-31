<script>
    import Transactions from "./Transactions.svelte";
    import Receive from "./Receive.svelte";
    import Send from "./Send.svelte";
    import { invoke } from "@tauri-apps/api";
    import Spinner from "../utils/Spinner.svelte";

    export let coin_index;
    export let coin_name;
    export let account_index;

    let TABS = {
        'tab-transactions': Transactions,
        'tab-send': Send,
        'tab-receive': Receive
    };

    let TOKEN_NAME = {
        0: 'BTC',
        1: 'TBTC'
    };

    let SATOSHI = 100000000;
    let tabContent = Transactions;

    const switchTabs = (tab_id) => {
        document.getElementById('tab-transactions').className = 'tab';
        document.getElementById('tab-send').className = 'tab';
        document.getElementById('tab-receive').className = 'tab';
        document.getElementById(tab_id).className = 'tab tab-active';
        tabContent = TABS[tab_id];
    }

    const getAccountBalance = () => {
        return invoke('get_account_balance', {coinTypeIndex: coin_index, accountIndex: account_index});
    }

</script>

<div class="p-5">
    <div class="pl-2">
        <div class="flex flex-row pb-1">
            <h1 class="text-2xl font-bold pb-2">{coin_name} #{account_index + 1}</h1>
        </div>
        {#await getAccountBalance()}
            <Spinner></Spinner>
        {:then balance}
            <h1 class="text-1xl font-bold text-gray-600">{balance / SATOSHI} {TOKEN_NAME[coin_index]}</h1>
        {/await}
    </div>
    <div class="pt-6">
        <div class="tabs tabs-boxed">
            <a id="tab-transactions" class="tab tab-active" on:click={() => switchTabs('tab-transactions')}>Transactions</a>
            <a id="tab-send" class="tab" on:click={() => switchTabs('tab-send')}>Send</a>
            <a id="tab-receive" class="tab" on:click={() => switchTabs('tab-receive')}>Receive</a>
        </div>
    </div>
    <svelte:component this={tabContent} coin_index={coin_index} account_index={account_index}></svelte:component>
</div>

