<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri';

	let chosen_method = 'Choose method';
	let generating = false;
	let options = ['Prefix only', 'Suffix only', 'Prefix + suffix'];
	let prefix = '';
	let message = '';
	let threads = 0;

	const choose_method = (method_name: string) => {
		if (!generating) chosen_method = method_name;
	};

	async function greet() {
		generating = true;
		message = await invoke('return_prefix', { input: prefix, trs: threads });
		generating = false;
	}
</script>

<main class="container">
	<div style="height:20px" />
	<details role="list">
		<summary aria-haspopup="listbox">{chosen_method}</summary>
		<ul role="listbox">
			{#each options as option}
				<li on:click={() => choose_method(option)} on:keydown>{option}</li>
			{/each}
		</ul>
	</details>
	{#if chosen_method == 'Prefix only' || chosen_method == 'Suffix only'}
		<input type="text" placeholder="Enter prefix/suffix" bind:value={prefix} />
	{:else if chosen_method == 'Prefix + suffix'}
		<input type="text" placeholder="Enter prefix" />
		<input type="text" placeholder="Enter suffix" />
	{/if}
	<input type="number" placeholder="Enter threads" bind:value={threads} />
	{#if generating}
		<button class="outline" on:click={() => (generating = false)}>Stop generation</button>
		<div><progress /></div>
	{:else}
		<button class="outline" on:click={greet}>Generate private key</button>
	{/if}
	<div>{message}</div>
</main>
