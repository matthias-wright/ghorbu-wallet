<script>
    import {clipboard} from "@tauri-apps/api";
    export let address;

    let copyButtonText = 'Copy to Clipboard';

    const copyToClipboard = (text) => {
        clipboard.writeText(text);
        document.getElementById('btn-copy').className = 'btn btn-success btn-sm';
        copyButtonText = 'Copied';
        setTimeout(function(){
            copyButtonText = 'Copy to Clipboard';
            document.getElementById('btn-copy').className = 'btn btn-secondary btn-sm';
        }, 1000);
    }
</script>

<div class="alert bg-base-100">
    <div>
        <div>
            <h3 class="font-bold">New Address</h3>
            <div class="pt-1">{address}</div>
        </div>
    </div>
    <div class="flex-none">
        <button id="btn-copy" class="btn btn-secondary btn-sm" on:click={() => copyToClipboard(address)}>{copyButtonText}</button>
    </div>
</div>
