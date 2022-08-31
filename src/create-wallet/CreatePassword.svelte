<script>
    import { createEventDispatcher } from 'svelte';

    const dispatch = createEventDispatcher();
    let password = '';
    let passwordConfirm = '';

    const next = () => {
        dispatch('done', {password: password});
    }

    document.onkeydown = function (event) {
        if (event.key === "Enter" && password !== '') {
            next();
        }
    }

</script>

<div class="hero min-h-screen bg-base-200">
    <div class="hero-content text-center">
        <div class="max-w-md">
            <h1 class="text-2xl font-bold">Create a password to protect your wallet.</h1>
            <div class="form-control py-6">
                <div class="">
                    <input type="password" placeholder="Enter a password..." class="input input-bordered w-full" bind:value={password} autofocus />
                </div>
                <div class="pt-4">
                    <input type="password" placeholder="Confirm the password..." class="input input-bordered w-full" bind:value={passwordConfirm} />
                </div>
                <div class="pt-4">
                    <button disabled={password === '' || password !== passwordConfirm} on:click={() => next()} class="btn btn-primary">Continue</button>
                </div>
            </div>
        </div>
    </div>
</div>
