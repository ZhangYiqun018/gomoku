import OpenAI from "openai";

const readStdin = async () => {
  const chunks = [];
  for await (const chunk of process.stdin) {
    chunks.push(chunk);
  }
  return Buffer.concat(chunks).toString("utf8");
};

const main = async () => {
  const raw = await readStdin();
  if (!raw) {
    throw new Error("No input received");
  }
  const payload = JSON.parse(raw);
  const {
    baseUrl,
    apiKey,
    model,
    temperature,
    topP,
    maxTokens,
    timeoutMs,
    system,
    user,
  } = payload;

  if (!apiKey) {
    throw new Error("Missing apiKey");
  }
  if (!model) {
    throw new Error("Missing model");
  }

  const client = new OpenAI({
    apiKey,
    baseURL: baseUrl && baseUrl.length ? baseUrl : undefined,
  });

  const schema = {
    name: "gomoku_move",
    schema: {
      type: "object",
      properties: {
        move: { type: "string" },
      },
      required: ["move"],
      additionalProperties: false,
    },
  };

  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), timeoutMs || 20000);

  let response;
  try {
    response = await client.chat.completions.create(
      {
        model,
        messages: [
          { role: "system", content: system },
          { role: "user", content: user },
        ],
        temperature,
        top_p: topP,
        max_tokens: maxTokens,
        response_format: { type: "json_schema", json_schema: schema },
      },
      { signal: controller.signal }
    );
  } catch (err) {
    response = await client.chat.completions.create(
      {
        model,
        messages: [
          { role: "system", content: system },
          { role: "user", content: user },
        ],
        temperature,
        top_p: topP,
        max_tokens: maxTokens,
        response_format: { type: "json_object" },
      },
      { signal: controller.signal }
    );
  } finally {
    clearTimeout(timeout);
  }

  const content = response?.choices?.[0]?.message?.content ?? "";
  if (!content) {
    throw new Error("Empty response");
  }
  process.stdout.write(content);
};

main().catch((err) => {
  process.stderr.write(String(err?.message || err));
  process.exit(1);
});
