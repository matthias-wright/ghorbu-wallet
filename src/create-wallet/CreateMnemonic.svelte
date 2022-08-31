
<script>
    import { createEventDispatcher } from 'svelte';
    import { invoke } from '@tauri-apps/api'

    const dispatch = createEventDispatcher();

    const generate = () => {
        invoke('generate_mnemonic')
            .then((response) => {
                dispatch('done', {mnemonic: response});
            })
    }

</script>

<div class="hero min-h-screen bg-base-200">
    <div class="hero-content text-center">
        <div class="max-w-md">
            <button on:click={() => generate()} class="btn btn-primary">Create new wallet</button>
        </div>
    </div>
</div>

