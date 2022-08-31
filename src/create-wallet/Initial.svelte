<script>
  import svelteLogo from '../assets/svelte.svg'
  import { createEventDispatcher } from 'svelte';
  import CreateMnemonic from "./CreateMnemonic.svelte";
  import ShowMnemonic from "./ShowMnemonic.svelte";
  import CreatePassphrase from "./CreatePassphrase.svelte"
  import CreatePassword from "./CreatePassword.svelte";
  import { invoke } from '@tauri-apps/api'

  const dispatch = createEventDispatcher();

  let mnemonic = null;
  let currentView = CreateMnemonic;

  const nextState = (e) => {
      if ('mnemonic' in e.detail) {
          mnemonic = e.detail.mnemonic;
          currentView = ShowMnemonic;
      } else if ('showed_mnemonic' in e.detail) {
          currentView = CreatePassphrase;
      } else if ('passphrase' in e.detail) {
          let passphrase = e.detail.passphrase;
          invoke('send_passphrase', {passphrase: passphrase})
              .then((response) => {});
          currentView = CreatePassword;
      } else if ('password' in e.detail) {
          let password = e.detail.password;
          invoke('create_master_key', {password: password})
              .then((response) => {});
          dispatch('done', {created_master_key: 1});
      }
  }

</script>


<svelte:component this={currentView} on:done={(e) => nextState(e)} mnemonic={mnemonic}></svelte:component>





