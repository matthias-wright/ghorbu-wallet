<script>
    import { createEventDispatcher } from 'svelte';
    import { invoke } from '@tauri-apps/api'

    const dispatch = createEventDispatcher();

    let password = '';
    let wrong_password_error = false;
    let io_error = false;
    let other_error = false;

    const validate = () => {
        invoke('load_master_key', {password: password})
        .then(() => {
            dispatch('done', {login_success: 1});
        })
        .catch((err) => {
            if (err === 'wrong_password_error') {
                wrong_password_error = true;
                io_error = false;
                other_error = false;
                password = '';
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

    document.onkeydown = function (event) {
        if (event.key === "Enter") {
            validate();
        }
    }

</script>

<div class="hero min-h-screen bg-base-200">
    <div class="hero-content text-center">
        <div class="max-w-md">
            <h1 class="text-2xl font-bold">Enter your wallet password.</h1>
            <div class="form-control py-6">
                <input type="password" placeholder="" class="input input-bordered" bind:value={password} autofocus />
                <div class="pt-4">
                    <button disabled={password === ''} on:click={() => validate()} class="btn btn-primary">Continue</button>
                </div>
            </div>
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
    </div>
</div>
