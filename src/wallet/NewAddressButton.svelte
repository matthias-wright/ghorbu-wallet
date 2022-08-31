<script>
    import { createEventDispatcher } from 'svelte';
    import { invoke } from '@tauri-apps/api';

    export let coin_index;
    export let account_index;

    const dispatch = createEventDispatcher();
    let password = '';
    let wrong_password_error = false;
    let io_error = false;
    let other_error = false;

    const getNewReceiveAddress = () => {
        invoke('get_new_receive_address', {coinTypeIndex: coin_index, accountIndex: account_index, password: password})
            .then((response) => {
                dispatch('clicked', {new_address: response});
            })
            .catch((err) => {
                if (err === 'wrong_password_error') {
                    wrong_password_error = true;
                    io_error = false;
                    other_error = false;
                } else if (err === 'io_error') {
                    wrong_password_error = false;
                    io_error = true;
                    other_error = false;
                } else {
                    wrong_password_error = false;
                    io_error = false;
                    other_error = true;
                }
            })
    }

    const newAddress = () => {
        window.location = '#popup-new-address';
        document.getElementById('input-password').focus();
    }

    document.onkeydown = function (event) {
        if (event.key === "Enter") {
            getNewReceiveAddress();
        }
    }
</script>

<button class="btn btn-secondary" on:click={() => newAddress()}>New Address</button>

<div class="modal" id="popup-new-address">
    <div class="modal-box">
        <h3 class="font-bold text-lg">Create new address</h3>
        <div class="form-control py-2">
            <input type="password" id="input-password" placeholder="Enter your wallet password..." class="input input-bordered" bind:value={password} />

            <div>
                {#if wrong_password_error}
                    <div class="alert alert-error shadow-lg">
                        <div>
                            <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current flex-shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
                            <span>Wrong password.</span>
                        </div>
                    </div>
                {:else if io_error}
                    <div class="alert alert-error shadow-lg">
                        <div>
                            <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current flex-shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
                            <span>Unable to load keys.</span>
                        </div>
                    </div>
                {:else if other_error}
                    <div class="alert alert-error shadow-lg">
                        <div>
                            <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current flex-shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
                            <span>Unable to load keys.</span>
                        </div>
                    </div>
                {/if}
            </div>
            <div class="flex flex-row justify-end">
                <div class="modal-action pr-1">
                    <a href="#" class="btn btn-secondary">Close</a>
                </div>
                <div class="modal-action pl-1">
                    <button class="btn btn-primary" disabled={password === ''} on:click={() => getNewReceiveAddress()}>Create</button>
                </div>
            </div>
        </div>
    </div>
</div>
