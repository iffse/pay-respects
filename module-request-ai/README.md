# Request AI Module

Adds AI suggestions to [pay-respects] when no rule matches.

[pay-respects]: https://codeberg.org/iff/pay-respects

## Configuration

The module reads the same `config.toml` as pay-respects:

```toml
[module.request-ai]
# Set to true or false. When unset and no API key is configured, the module
# asks whether to use the built-in key.
enabled = true

# Your API key. Leave unset to use the built-in key.
api_key = "..."

# Any OpenAI-compatible endpoint works:
# - https://api.openai.com/v1/chat/completions: OpenAI
# - https://api.groq.com/openai/v1/chat/completions: GroqCloud
# - http://localhost:11434/v1/chat/completions: Local Ollama
api_url = "https://api.openai.com/v1/chat/completions"

# Model name. Reasoning models work too.
model = "gpt-4o-mini"

# Extra JSON fields for the request, for example `"temperature" = 0.5`.
extra = '"temperature" = 0.5'

# JSON object for the `extra_body` field.
extra_body = '{"chat_template_kwargs": {"enable_thinking": false}}'

# Language for the explanation. Defaults to the system locale.
locale = "en-US"

# Extra text added before the prompt.
additional_prompt = "User's environment is Zsh running in Arch Linux."
```

When `enabled` is unset:

- If an API key is configured in `config.toml` or `_PR_AI_API_KEY`, the module
	uses that key.
- Otherwise the module asks whether to use the built-in key. The first option
	applies to this request. The `(always)` options also write `enabled = true`
	or `enabled = false` to your config.
- In non-interactive modes, or when stderr is not a terminal, the module skips
	the prompt and sends no request.

Environment variables still work and override `config.toml`:

- `_PR_AI_API_KEY`: API key
- `_PR_AI_URL`: OpenAI-compatible endpoint
- `_PR_AI_MODEL`: Model name
- `_PR_AI_EXTRA`: Extra JSON fields, for example `"temperature" = 0.5`
- `_PR_AI_EXTRA_BODY`: JSON object for `extra_body`
- `_PR_AI_DISABLE`: Any value disables the module
- `_PR_AI_LOCALE`: Language for the explanation
- `_PR_AI_ADDITIONAL_PROMPT`: Extra text added before the prompt

These build-time variables set fallback values:

- `_DEF_PR_AI_API_KEY`
- `_DEF_PR_AI_URL`
- `_DEF_PR_AI_MODEL`

If no compile-time defaults are set, pay-respects uses its own key after you
agree to use it. The proxy filters requests for abuse, then forwards them to a
provider that does not train on your data. The service is free and may stop at
any time. Donations are welcome:

<div>
	<a
		href="https://liberapay.com/iff/donate"
		target="_blank"
		rel="noreferrer"
		><img
			src="https://liberapay.com/assets/widgets/donate.svg"
			alt="Donate using Liberapay"
		/></a
	>
	<a href="https://ko-fi.com/iffse" target="_blank" rel="noreferrer"
		><img
			height='30'
			src="https://www.vectorlogo.zone/logos/ko-fi/ko-fi-ar21.svg"
			alt="Donate using Ko-fi"
			style="height: 30px;"
		/></a
	>
	<br />
	<a href="https://iffse.eu.org/stripe" target="_blank" rel="noreferrer"
		><img
			height='30'
			src="https://cdn.brandfolder.io/KGT2DTA4/at/8vbr8k4mr5xjwk4hxq4t9vs/Stripe_wordmark_-_blurple.svg"
			alt="Donate using Stripe"
			style="height: 30px;"
		/></a
	>
	<a
		href="https://www.paypal.com/donate/?hosted_button_id=QN7Z7ZHRAAFZL"
		target="_blank"
		rel="noreferrer"
		><img
			height='30'
			src="https://upload.wikimedia.org/wikipedia/commons/b/b5/PayPal.svg"
			alt="Donate using PayPal"
			style="height: 25px; margin-bottom: 3px;"
		/></a
	>
</div>

## Advanced Usage

Add context as a comment at the end of a command. Bash and Zsh need interactive
comments enabled.

```sh
rustup # how do I set up the nightly toolchain?
```

A comment alone works too:

```sh
# cache git credentials for 3 hours
```
