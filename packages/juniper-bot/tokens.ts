import { load } from "https://deno.land/std@0.224.0/dotenv/mod.ts"

await load({
	export: true,
})

const OLLAMA_ENDPOINT = Deno.env.get("OLLAMA_ENDPOINT")
const DISCORD_TOKEN = Deno.env.get("DISCORD_TOKEN")

export { OLLAMA_ENDPOINT, DISCORD_TOKEN }