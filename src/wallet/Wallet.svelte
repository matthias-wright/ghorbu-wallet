<script>
    import svelteLogo from '../assets/svelte.svg'
    import { onMount } from 'svelte';
    import NewAccount from "./NewAccount.svelte";
    import AccountOverview from "./AccountOverview.svelte";
    import WelcomeScreen from "./WelcomeScreen.svelte";
    import { invoke } from '@tauri-apps/api'

    let accounts = null;
    let newAccountModal = false;
    let accountOverview = null;

    let currentCoinIndex = 0;
    let currentCoinName = '';
    let currentAccountIndex = 0;

    function getAccountsOverview() {
         invoke('get_accounts_overview')
            .then((response) => {
                if (response !== '') {
                    accounts = JSON.parse(response);
                }
            })
    }

    onMount(async () => {
        getAccountsOverview();
        accountOverview = WelcomeScreen;
    });

    const accountCreated = () => {
        newAccountModal = false;
        getAccountsOverview();
    }

    const loadAccountOverview = (coinIndex, coinName, accountIndex) => {
        getAccountsOverview();
        currentCoinIndex = coinIndex;
        currentCoinName = coinName;
        currentAccountIndex = accountIndex;
        accountOverview = AccountOverview;
    }
</script>

<div>
    {#if accounts !== null}
        <div class="flex flex-row h-screen">
            <div>
                <div class="flex flex-row pt-3 pl-3">
                    <div class="w-32 pl-2">
                        <h1 class="text-2xl font-bold">Accounts</h1>
                    </div>
                    <div class="tooltip tooltip-bottom tooltip-primary" data-tip="Create new account">
                        <button on:click={() => newAccountModal = true} class="btn btn-primary btn-square btn-sm">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor"> <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm1-11a1 1 0 10-2 0v2H7a1 1 0 100 2h2v2a1 1 0 102 0v-2h2a1 1 0 100-2h-2V7z" clip-rule="evenodd" /> </svg>
                        </button>
                    </div>
                </div>
                <ul class="menu bg-base-100 w-56 p-2 rounded-box">
                    {#each accounts.purpose.coin_types as coin_type (coin_type.index)}
                        {#each coin_type.accounts as account (account.index)}
                            <li>
                                <a class="text-1xl font-bold" on:click={() => loadAccountOverview(coin_type.index, coin_type.name, account.index)}>{coin_type.name} #{account.index + 1}</a>
                            </li>
                        {/each}
                    {/each}
                </ul>
            </div>
            <div class="flex-1 w-full h-full bg-base-200">
                {#key currentCoinName}
                    {#key currentAccountIndex}
                        <svelte:component this={accountOverview} coin_index={currentCoinIndex} coin_name={currentCoinName} account_index={currentAccountIndex} ></svelte:component>
                    {/key}
                {/key}
            </div>
        </div>
    {/if}

    <svelte:component this={NewAccount} on:close={(event) => accountCreated()} open={newAccountModal} accounts={accounts}></svelte:component>
</div>





