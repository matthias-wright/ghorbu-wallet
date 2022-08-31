
<script>
    import { createEventDispatcher } from 'svelte';

    const dispatch = createEventDispatcher();

    let passphrase = '';
    let passphraseConfirm = '';

    const next = () => {
        dispatch('done', {passphrase: passphrase});
    }

    document.onkeydown = function (event) {
        if (event.key === "Enter") {
            next();
        }
    }

</script>

<div class="hero min-h-screen bg-base-200">
    <div class="hero-content text-center">
        <div class="max-w-md">
            <h1 class="text-2xl font-bold">Optional: Protect your seed phrase with a passphrase. Leave the input field empty for no passphrase.</h1>
            <div class="form-control py-6">
                <div class="">
                    <input type="text" placeholder="Enter a passphrase..." class="input input-bordered w-full" bind:value={passphrase} autofocus />
                </div>
                <div class="pt-4">
                    <input type="text" placeholder="Confirm the passphrase..." class="input input-bordered w-full" bind:value={passphraseConfirm} />
                </div>
                <div class="pt-4">
                    <button disabled={passphrase !== passphraseConfirm} on:click={() => next()} class="btn btn-primary">Continue</button>
                </div>
            </div>
        </div>
    </div>
</div>
