import { CommandClient } from "https://deno.land/x/harmony@v2.9.1/mod.ts"
import { load } from "https://deno.land/std@0.224.0/dotenv/mod.ts"
import { Ollama } from "https://esm.sh/ollama@0.5.15"
import { green } from "https://deno.land/std@0.224.0/fmt/colors.ts"

await load({
	export: true
})

const client = new CommandClient({
	token: Deno.env.get("DISCORD_TOKEN")!,
	prefix: "!",
	intents: ["GUILDS", "GUILD_MESSAGES"]
})

client.on("ready", () => {
	console.log(`${green("[INFO]")} Logged in as ${client.user?.tag}`)
})

client.slash.commands.create({
	name: "query",
	description: "Send a query to Ollama",
	options: [
		{
			name: "prompt",
			description: "The prompt to send",
			type: 3,
			required: true
		}
	]
}, Deno.env.get("DISCORD_GUILD_ID")!)

const ollama = new Ollama({
	host: Deno.env.get("OLLAMA_HOST") || "http://localhost:11434"
})

client.slash.handle("query", async interaction => {
	const prompt = interaction.data.options?.find(o => o.name === "prompt")?.value as string

	await interaction.respond({ type: 5 })

	try {
		const result = await ollama.generate({ model: "juniper-llama", prompt, stream: false })
		const chunks = chunk(result.response, 1900)

		await interaction.send(`\n${chunks[0]}`)

		for (let i = 1; i < chunks.length; i++) {
			await interaction.send(chunks[i])
		}
	} catch (err) {
		await interaction.send(`❌ Error: ${err instanceof Error ? err.message : "Unknown error"}`)
	}
})

function chunk(str: string, size: number): string[] {
	const chunks = []
	for (let i = 0; i < str.length; i += size) {
		chunks.push(str.slice(i, i + size))
	}
	return chunks
}

await client.connect()
