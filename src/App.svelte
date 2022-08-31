<script>
  import Initial from "./create-wallet/Initial.svelte";
  import Wallet from "./wallet/Wallet.svelte";
  import Login from "./wallet/Login.svelte";
  import { invoke } from '@tauri-apps/api'
  import { onMount } from 'svelte';

  onMount(async () => {
      invoke('does_master_key_exist')
      .then((response) => {
          if (response) {
              currentView = Login;
          } else {
              currentView = Initial;
          }
      })
  });

  let currentView = Wallet;
  let initial = false;
  if (initial) {
      currentView = Initial;
  }

  const nextState = (e) => {
    if ('login_success' in e.detail) {
        currentView = Wallet;
    } else if ('created_master_key' in e.detail) {
        currentView = Wallet;
    }
  }

</script>

<main data-theme="cupcake">
    <div class="flex flex-col h-screen">
        <div class="flex-1 h-full">
            <svelte:component this={currentView} on:done={(e) => nextState(e)}></svelte:component>
        </div>
    </div>
</main>

<style>
    main {
        overflow: hidden;
    }
</style>
