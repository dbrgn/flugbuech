<script lang="ts">
  export let id: string;
  export let label: string;
  export let required = false;
  export let error = false;
  export let icon: string;
  export let value: string;

  // TODO: Should this be used for other forms as well?
</script>

<div class="field">
  <label class="label" for={id}>{label}</label>
  <div class="control has-icons-left">
    <input {id} class="input" class:error type="password" bind:value {required} />
    <span class="icon is-small is-left">
      <i class="fa-solid {icon}"></i>
    </span>
  </div>
</div>

<style>
  .field input.error {
    border: 1px solid #ff3860;
  }
</style>
