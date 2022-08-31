
<script>
    import { invoke } from '@tauri-apps/api'
    import { onMount } from 'svelte';
    import NewAddressButton from "./NewAddressButton.svelte";
    import ShowNewAddress from "./ShowNewAddress.svelte";
    import Spinner from "../utils/Spinner.svelte";

    export let coin_index;
    export let account_index;

    let showNewAddressView = NewAddressButton;
    let newAddress = '';
    let addresses = null;

    const showNewAddress = (e) => {
        newAddress = e.detail.new_address;
        showNewAddressView = ShowNewAddress;
    }

    const getAllReceiveAddresses = () => {
        invoke('get_all_receive_addresses_marked', {coinTypeIndex: coin_index, accountIndex: account_index})
            .then((response) => {
                addresses = response;
            })
            .catch((err) => {

            })
    }

    const getAllReceiveAddressesX = () => {
        return invoke('get_all_receive_addresses_marked', {coinTypeIndex: coin_index, accountIndex: account_index});
    }

    onMount(async () => {
        getAllReceiveAddresses();
    });

</script>


<div class="pt-4 pl-2">
    <div class="pb-2">
        <svelte:component this={showNewAddressView} address={newAddress} coin_index={coin_index} account_index={account_index} on:clicked={(e) => showNewAddress(e)}></svelte:component>
    </div>
</div>
{#await getAllReceiveAddressesX()}
    <div class="pl-4 pt-8">
        <Spinner height=9 width=9></Spinner>
    </div>
{:then addresses}
    <div class="pt-4 pl-2">
        <div class="table-wrp block max-h-96 overflow-x-auto">
            <table class="table w-full">
                <thead class="border-b sticky top-0">
                <tr>
                    <th>Address</th>
                    <th>Status</th>
                </tr>
                </thead>
                <tbody>
                    {#each addresses as address}
                        <tr>
                            <td>{address.address}</td>
                            <td>
                                <div class={address.used ? 'badge badge-warning gap-1' : 'badge badge-success gap-1'}>
                                    {address.used ? 'used' : 'unused'}
                                </div>
                            </td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
    </div>
{:catch error}
    <div class="pt-8">
        <div class="alert alert-error shadow-lg">
            <div>
                <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current flex-shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
                <span>Unable to load addresses.</span>
            </div>
        </div>
    </div>
{/await}

<style>
    .table-wrp  {
        overflow-y: auto;
        display:block;
    }
    thead{
        position:sticky;
        top:0
    }
</style>
