export default function NotFound() {
	return (
		<section class="flex flex-col gap-8 justify-center items-center size-full bg-gray-primary">
			<div class="size-32 not-found">{" "}</div>
			<div class="flex flex-col gap-2">
				<h1 class="text-white text-2xl rounded-lg px-2 font-mono font-bold text-center">
					Uh Oh! <br />
				</h1>
				<span class="text-white max-w-[25vw] opacity-50 text-center">
					It seems so we cannot find the page you were looking for,
					sorry!
				</span>
			</div>
		</section>
	);
}
