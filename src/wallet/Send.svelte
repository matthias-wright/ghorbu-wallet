<script>
    import { onMount } from 'svelte';
    import { invoke } from "@tauri-apps/api";
    import Spinner from "../utils/Spinner.svelte";

    export let coin_index;
    export let account_index;

    let TOKEN_NAME = {
        0: 'BTC',
        1: 'TBTC'
    };

    let MINIMUM_AMOUNT = 0.00000001;
    let SATOSHI = 100000000;

    let address = '';
    let amountStr = '';
    let amount = 0;
    let fees = null;
    let fee = 1;
    let password = '';
    let errorOccurred = false;
    let errorMessage = '';
    let amountError = '';
    let addressError = '';
    let processingTransaction = false;
    let totalAmountSent = 0;

    let addressInputClass = 'input input-bordered';
    let amountInputClass = 'input input-bordered w-full';
    let amountValid = false;
    let addressValid = false;

    const sendTransaction = async () => {
        processingTransaction = true;
        invoke('send_transaction', {
            coinTypeIndex: coin_index,
            accountIndex: account_index,
            address: address,
            amount: amount,
            fee: fee,
            password: password})
            .then((response) => {
                address = '';
                amountStr = '';
                totalAmountSent = response / SATOSHI;
                errorOccurred = false;
                errorMessage = '';
                addressInputClass = 'input input-bordered';
                amountInputClass = 'input input-bordered w-full';
                window.location = '#popup-success';
            })
            .catch((err) => {
                processingTransaction = false;
                if (err === 'wrong_password_error') {
                    errorOccurred = true;
                    errorMessage = 'Wrong password.';
                    password = '';
                } else if (err === 'io_error') {
                    errorOccurred = true;
                    errorMessage = 'Unable to load keys..';
                } else if (err === 'send_tx_error') {
                    errorOccurred = true;
                    errorMessage = 'Sending the transaction failed.';
                } else if (err === 'send_tx_error') {
                    errorOccurred = true;
                    errorMessage = 'Creating the transaction failed.';
                } else if (err === 'balance_insufficient') {
                    errorOccurred = true;
                    errorMessage = 'Not enough funds available.';
                } else if (err === 'max_input_count_exceeded') {
                    errorOccurred = true;
                    errorMessage = 'Maximum number of inputs exceeded.';
                } else {
                    errorOccurred = true;
                    errorMessage = 'An error occurred.';
                }
            })
    }

    const validateAddress = () => {
        invoke('validate_address', {address: address, coinTypeIndex: coin_index})
            .then(() => {
                addressInputClass = 'input input-bordered input-success';
                addressValid = true;
                addressError = '';
            })
            .catch((err) => {
                addressInputClass = 'input input-bordered input-error';
                addressValid = false;
                addressError = err;
            })
    }

    const validateAmount = () => {
        let rgx = /^[0-9]*\.?[0-9]*$/;
        if (amountStr === '') {
            amountInputClass = 'input input-bordered w-full';
            amountValid = false;
            amountError = '';
        } else if (amountStr.match(rgx) === null) {
            // invalid input
            amountInputClass = 'input input-bordered w-full input-error';
            amountValid = false;
            amountError = 'Amount must be a number';
        } else {
            // convert bitcoin -> satoshi
            amount = Math.round(parseFloat(amountStr) * SATOSHI);
            if (amount < MINIMUM_AMOUNT) {
                amountInputClass = 'input input-bordered w-full input-error';
                amountValid = false;
                amountError = 'Amount must be at least ' + MINIMUM_AMOUNT;
            } else {
                // valid input
                amountInputClass = 'input input-bordered w-full input-success';
                amountValid = true;
                amountError = '';
            }
        }
    }

    const getRecommendedFees = () => {
        invoke('get_recommended_fees', {coinTypeIndex: coin_index})
            .then((response) => {
                fees = response;
            })
    }

    const switchTabs = (tab_id) => {
        document.getElementById('fastestFee').className = 'tab';
        document.getElementById('halfHourFee').className = 'tab';
        document.getElementById('hourFee').className = 'tab';
        document.getElementById('economyFee').className = 'tab';
        document.getElementById('minimumFee').className = 'tab';
        document.getElementById(tab_id).className = 'tab tab-active';
        fee = fees[tab_id];
    }

    const openSendModal = () => {
        window.location = "#popup-send";
        document.getElementById('input-password').focus();
    }

    const closeSendModal = () => {
        password = '';
        errorOccurred = false;
        errorMessage = '';
    }

    document.onkeydown = function (event) {
        if (event.key === "Enter" && addressValid && amountValid) {
            sendTransaction();
        }
    }

    onMount(async () => {
        getRecommendedFees();
        totalAmountSent = 0;
        errorOccurred = false;
        errorMessage = '';
    });

</script>

<div class="pt-4 pl-2">
    <div class="form-control py-2">
        <span class="upper-case text-gray-600 pb-2">Address</span>
        <input type="text" placeholder="" class={addressInputClass} bind:value={address} on:input={() => validateAddress()} autofocus />
        <span contenteditable class="text-xs text-red-800">{addressError}</span>

        <span class="upper-case text-gray-600 pb-2 pt-4">Amount</span>
        <label class="input-group">
            <input type="text" placeholder="" class={amountInputClass} bind:value={amountStr} on:input={() => validateAmount()} />
            <span>{TOKEN_NAME[coin_index]}</span>
        </label>
        <span contenteditable class="text-xs text-red-800">{amountError}</span>

        <div class="pt-4">
            <div class="pr-6 pb-2">
                <span class="upper-case text-gray-600">Fee</span>
            </div>
            <div class="tabs tabs">
                <a id="fastestFee" class="tab" on:click={() => switchTabs('fastestFee')}>Fastest</a>
                <a id="halfHourFee" class="tab tab-active" on:click={() => switchTabs('halfHourFee')}>Half Hour</a>
                <a id="hourFee" class="tab" on:click={() => switchTabs('hourFee')}>Hour</a>
                <a id="economyFee" class="tab" on:click={() => switchTabs('economyFee')}>Economy</a>
                <a id="minimumFee" class="tab" on:click={() => switchTabs('minimumFee')}>Minimum</a>
            </div>
        </div>

        <div class="flex flex-row pt-9 justify-end">
            <div>
                <button disabled={!(addressValid && amountValid)} on:click={() => openSendModal()} class="btn btn-primary btn-wide">Send</button>
            </div>
        </div>

    </div>
</div>

<div class="modal" id="popup-send">
    <div class="modal-box">
        <h3 class="font-bold text-lg">Send</h3>
        <div class="form-control py-2">
            <input type="password" id="input-password" placeholder="Enter your wallet password..." class="input input-bordered" bind:value={password} />
            <div>
                {#if errorOccurred}
                    <div class="alert alert-error shadow-lg">
                        <div>
                            <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current flex-shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
                            <span>{errorMessage}</span>
                        </div>
                    </div>
                {/if}
            </div>
            <div class="flex flex-row justify-between">
                {#if processingTransaction}
                    <div class="pt-8">
                        <Spinner></Spinner>
                    </div>
                {:else}
                    <span></span>
                {/if}
                <div class="flex flex-row justify-end">
                    <div class="modal-action pr-1">
                        <a href="#" class="btn btn-secondary" on:click={() => closeSendModal()}>Close</a>
                    </div>
                    <div class="modal-action pl-1">
                        <button class="btn btn-primary" disabled={password === ''} on:click={() => sendTransaction()}>Confirm</button>
                    </div>
                </div>
            </div>
        </div>
    </div>
</div>

<div class="modal" id="popup-success">
    <div class="modal-box">
        <h3 class="font-bold text-lg">Confirmation</h3>
        <div>
            <div class="pt-4">
                <span>The transaction was successfully broadcasted. Total amount sent (incl. fees): {totalAmountSent} {TOKEN_NAME[coin_index]}.</span>
            </div>
        </div>
        <div class="flex flex-row justify-end">
            <div class="modal-action pl-1">
                <a href="#" class="btn btn-primary">Ok</a>
            </div>
        </div>
    </div>
</div>

<style>
    .upper-case {
        text-transform: uppercase;
        font-weight: 700;
        font-size: 1rem;
    }
</style>