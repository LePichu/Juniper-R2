import { CommandClient } from "https://deno.land/x/harmony@v2.9.1/mod.ts"
import { load } from "https://deno.land/std@0.224.0/dotenv/mod.ts"
import { Ollama } from "https://esm.sh/ollama"

// Load .env
await load()

const client = new CommandClient({
	token: Deno.env.get("DISCORD_TOKEN")!,
	prefix: "!",
	intents: ["GUILDS", "GUILD_MESSAGES"]
})

client.on("ready", () => {
	console.log(`Logged in as ${client.user?.tag}`)
})

// Register slash command manually per guild
client.slash.commands.create({
	name: "query",
	description: "Send a query to Ollama",
	options: [
		{
			name: "model",
			description: "The model to use",
			type: 3, // STRING
			required: true
		},
		{
			name: "prompt",
			description: "The prompt to send",
			type: 3, // STRING
			required: true
		}
	]
}, Deno.env.get("DISCORD_GUILD_ID")!) // Use GUILD_ID for testing

const ollama = new Ollama({
	host: Deno.env.get("OLLAMA_HOST") || "http://localhost:11434"
})

// Handle command
client.slash.handle("query", async interaction => {
	const model = interaction.data.options?.find(o => o.name === "model")?.value as string
	const prompt = interaction.data.options?.find(o => o.name === "prompt")?.value as string

	await interaction.respond({ type: 5 }) // ACK the interaction (DEFER)

	try {
		const result = await ollama.generate({ model, prompt, stream: false })
		const chunks = chunk(result.response, 1900)

		await interaction.send(`**Model**: ${model}\n**Response**:\n${chunks[0]}`)

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

await client.connect().then(() => console.log("Started"))
