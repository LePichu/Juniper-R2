import { createEffect, createSignal, For, onMount } from "solid-js";
import { Ollama } from "ollama";
import { marked } from "marked";

function App() {
    const [input, setInput] = createSignal("");
    const [messages, setMessages] = createSignal([]);
    const [isLoading, setIsLoading] = createSignal(false);
    const [model, setModel] = createSignal("llama3");
    const [error, setError] = createSignal(null);

    const ollama = new Ollama({
        host: "http://localhost:11434",
    });

    onMount(() => {
        const savedMessages = localStorage.getItem("ollama-chat-messages");
        const savedModel = localStorage.getItem("ollama-chat-model");

        if (savedMessages) {
            try {
                setMessages(JSON.parse(savedMessages));
            } catch (err) {
                console.error("Failed to parse saved messages:", err);
            }
        }

        if (savedModel) {
            setModel(savedModel);
        }
    });

    const saveMessages = (newMessages) => {
        setMessages(newMessages);
        localStorage.setItem(
            "ollama-chat-messages",
            JSON.stringify(newMessages),
        );
    };

    const saveModel = (newModel) => {
        setModel(newModel);
        localStorage.setItem("ollama-chat-model", newModel);
    };

    const handleSubmit = async (e) => {
        e.preventDefault();

        const userMessage = input();
        if (!userMessage.trim()) return;

        const updatedMessages = [...messages(), {
            role: "user",
            content: userMessage,
        }];
        saveMessages(updatedMessages);
        setInput("");
        setIsLoading(true);
        setError(null);

        try {
            const newMessage = { role: "assistant", content: "" };
            saveMessages([...updatedMessages, newMessage]);

            const response = await ollama.chat({
                model: "juniper-llama",
                messages: [...updatedMessages],
                stream: true,
            });

            for await (const chunk of response) {
                setMessages((prev) => {
                    const updatedMessages = [...prev];
                    const lastMessage =
                        updatedMessages[updatedMessages.length - 1];
                    lastMessage.content += chunk.message.content || "";
                    localStorage.setItem(
                        "ollama-chat-messages",
                        JSON.stringify(updatedMessages),
                    );
                    return updatedMessages;
                });
            }
        } catch (err) {
            console.error("Error calling Ollama API:", err);
            setError(
                `Error: ${err.message || "Failed to get response from Ollama"}`,
            );
            saveMessages(updatedMessages);
        } finally {
            setIsLoading(false);
        }
    };

    const clearChat = () => {
        saveMessages([]);
    };

    createEffect(() => {
        const messagesLength = messages().length;
        if (messagesLength > 0) {
            const chatContainer = document.getElementById("chat-container");
            chatContainer.scrollTop = chatContainer.scrollHeight;
        }
    });

    return (
        <div class="flex flex-col mx-auto text-white bg-gray-primary h-full">
            {/* <header class="bg-gray-800 text-white p-4">
                <div class="container mx-auto flex justify-between items-center">
                    <h1 class="text-xl font-bold">Ollama Chat</h1>
                    <div class="flex items-center gap-4">
                        <select
                            value={model()}
                            onChange={(e) => saveModel(e.target.value)}
                            class="bg-gray-700 text-white px-2 py-1 rounded"
                        >
                            <option value="llama3">Llama 3</option>
                            <option value="mistral">Mistral</option>
                            <option value="llama2">Llama 2</option>
                            <option value="gemma">Gemma</option>
                        </select>
                        <button
                            onClick={clearChat}
                            class="bg-red-600 hover:bg-red-700 px-3 py-1 rounded text-sm"
                        >
                            Clear Chat
                        </button>
                    </div>
                </div>
            </header> */}

            <main class="flex-1 overflow-hidden container mx-auto flex flex-col p-4">
                <div
                    id="chat-container"
                    class="flex-1 overflow-y-auto mb-4 p-4"
                >
                    <For each={messages()}>
                        {(message) => (
                            <div
                                class={`mb-4 ${message.role === "user"
                                        ? "text-right"
                                        : "text-left"
                                    }`}
                            >
                                <div
                                    class={`inline-block px-4 py-2 rounded-lg max-w-3xl text-left ${message.role === "user"
                                            ? "bg-[#2e2e2e] text-white ml-auto"
                                            : "bg-branding text-gray-800"
                                        }`}
                                >
                                    {message.role === "assistant"
                                        ? (
                                            <div
                                                class="markdown-content"
                                                innerHTML={marked.parse(
                                                    message.content,
                                                )}
                                            >
                                            </div>
                                        )
                                        : (
                                            <div class="whitespace-pre-wrap">
                                                {message.content}
                                            </div>
                                        )}
                                </div>
                            </div>
                        )}
                    </For>
                    {isLoading() && (
                        <div class="flex items-center text-gray-500">
                            <div class="dot-flashing"></div>
                        </div>
                    )}
                    {error() && (
                        <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
                            {error()}
                        </div>
                    )}
                </div>

                <form onSubmit={handleSubmit} class="flex gap-2">
                    <input
                        type="text"
                        value={input()}
                        onInput={(e) => setInput(e.target.value)}
                        placeholder="Ask something..."
                        class="flex-1 p-2 rounded focus:outline-none bg-[#2c2c2c]"
                        disabled={isLoading()}
                    />
                    <button
                        type="submit"
                        class="px-4 py-2 rounded aspect-square bg-branding"
                        disabled={isLoading() || !input().trim()}
                    >
                        <img src="/send--alt--filled.svg" class="size-8 invert" />
                    </button>
                </form>
            </main>
        </div>
    );
}

export default App;
