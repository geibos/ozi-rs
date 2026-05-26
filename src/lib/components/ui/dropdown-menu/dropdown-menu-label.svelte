<script lang="ts">
	import { DropdownMenu as DropdownMenuPrimitive } from "bits-ui";
	import { cn } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		class: className,
		inset,
		...restProps
	}: DropdownMenuPrimitive.GroupHeadingProps & { inset?: boolean } = $props();
</script>

<DropdownMenuPrimitive.GroupHeading
	bind:ref
	data-slot="dropdown-menu-label"
	data-inset={inset ? "" : undefined}
	class={cn("px-2 py-1.5 text-xs font-medium data-[inset]:pl-8", className)}
	{...restProps}
/>
