<script>
    import { createEventDispatcher } from 'svelte';
    import { onMount, tick, afterUpdate } from 'svelte';
    import { invoke } from '@tauri-apps/api';

    const dispatch = createEventDispatcher();
    export let open = false;
    export let accounts;
    let password = '';
    let coin_type_index = "-1";
    let wrong_password_error = false;
    let io_error = false;
    let other_error = false;

    let popup_title = '';
    let popup_message = '';

    onMount(async () => {
        await tick();
    })

    afterUpdate(async () => {
        await tick();
    })

    function close() {
        password = '';
        coin_type_index = "-1";
        open = false;
        dispatch('close', {});
    }

    const validate = () => {
        invoke('create_new_account', {coinTypeIndex: coin_type_index, password: password})
            .then(() => {
                password = '';
                coin_type_index = "-1";
                dispatch('close', {});
            })
            .catch((err) => {
                if (err === 'wrong_password_error') {
                    wrong_password_error = true;
                    io_error = false;
                    other_error = false;
                    window.location = '#popup-modal';
                    password = '';
                    popup_title = 'Warning';
                    popup_message = 'The password is incorrect. Please try again.';
                } else if (err === 'io_error') {
                    wrong_password_error = false;
                    io_error = true;
                    other_error = false;
                    window.location = '#popup-modal';
                    popup_title = 'Warning';
                    popup_message = 'Unable to load keys.';
                } else {
                    wrong_password_error = false;
                    io_error = false;
                    other_error = true;
                    window.location = '#popup-modal';
                    popup_title = 'Warning';
                    popup_message = 'Unable to load keys.';
                }
            })
    }

    document.onkeydown = function (event) {
        if (event.key === "Enter") {
            validate();
        }
    }

</script>

{#if accounts !== null}
<div class="modal" class:modal-open={open}>
    <div class="modal-box">
        <h3 class="font-bold text-lg pb-2">Create new account</h3>
        <select bind:value={coin_type_index} class="select select-primary w-full max-w-xs">
            <option value="-1" selected disabled>Select a coin</option>
            {#each accounts.purpose.coin_types as coin_type (coin_type.index)}
                <option value={coin_type.index}>{coin_type.name}</option>
            {/each}
        </select>

        <div class="form-control py-2">
            <input type="password" placeholder="Enter your wallet password..." class="input input-bordered" bind:value={password} />

            <div class="flex flex-row justify-end">
                <div class="modal-action pr-1">
                    <button class="btn btn-secondary" on:click={() => close()}>Close</button>
                </div>

                <div class="modal-action pl-1">
                    <button class="btn btn-primary" disabled={password === '' || coin_type_index === "-1"} on:click={() => validate()}>Create</button>
                </div>
            </div>
        </div>

    </div>
</div>
{/if}

<div class="modal" id="popup-modal">
    <div class="modal-box">
        <h3 class="font-bold text-lg">{popup_title}</h3>
        <p class="py-4">{popup_message}</p>
        <div class="modal-action">
            <a href="#" class="btn btn-secondary">Ok</a>
        </div>
    </div>
</div>
