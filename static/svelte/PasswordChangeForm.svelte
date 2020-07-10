<script lang="typescript">
    import { onMount } from 'svelte';
    import FormInputField from './FormInputField.svelte';

    onMount(() => {
    });
</script>

<form action="" method="post">

<FormInputField id="current" label="Current Password" icon="fa-key" required=true />
<FormInputField id="new1" label="New Password" icon="fa-asterisk" required=true />
<FormInputField id="new2" label="New Password (repeat)" icon="fa-asterisk" required=true />

<div class="content control submitcontrols">
    <button type="submit" class="button is-info">Submit</button>
</div>

<p class="content"><small><em>* Required fields</em></small></p>

</form>
