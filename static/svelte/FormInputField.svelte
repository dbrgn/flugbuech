<script lang="typescript">
    export let id;
    export let label;
    export let required = false;
    export let icon;
</script>

<label class="label" for={id}>{label}{#if required} *{/if}</label>
<div class="field">
    <div class="control has-icons-left">
        <input class="input" type="password" {id} name={id} {required}>
        <div class="icon is-small is-left">
            <i class="fas {icon}"></i>
        </div>
    </div>
</div>
