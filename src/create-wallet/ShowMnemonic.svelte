<script>
    import { createEventDispatcher } from 'svelte';

    const dispatch = createEventDispatcher();

    export let mnemonic;

    const next = () => {
        dispatch('done', {showed_mnemonic: null});
    }

</script>

<div class="hero min-h-screen bg-base-200">
    <div class="hero-content text-center">
        <div class="max-w-md">
            <h1 class="text-3xl font-bold">Seed phrase:</h1>
            <p class="py-6">{mnemonic}</p>

            <div class="alert alert-warning">
                <div>
                    <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current flex-shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" /></svg>
                    <span>Write down or memorize the seed phrase.</span>
                </div>
            </div>
            <div class="m-10"><button on:click={() => next()} class="btn btn-primary">Continue</button></div>
        </div>
    </div>
</div>


