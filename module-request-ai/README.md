# Request AI Module

Module for [pay-respects](https://codeberg.org/iff/pay-respects) to request AI for suggestions.

## Configurations

The module reads the same `config.toml` as pay-respects:

```toml
[module.request-ai]
# Enable AI suggestions.
# If unset, and no API key is configured, the module asks whether to use the
# built-in API key.
enabled = true

# Your own API key. Leave unset to use the built-in API key.
api_key = "..."

# Any OpenAI compatible URL can be used, e.g.:
# - https://api.openai.com/v1/chat/completions: OpenAI ChatGPT
# - https://api.groq.com/openai/v1/chat/completions: GroqCloud
# - http://localhost:11434/v1/chat/completions: Local Ollama
api_url = "https://api.openai.com/v1/chat/completions"

# Model used. Reasoning models are also supported
model = "gpt-4o-mini"

# Extra Json field requests, e.g. `"temperature" = 0.5`
extra = '"temperature" = 0.5'

# A Json object for the `extra_body` field
extra_body = '{"chat_template_kwargs": {"enable_thinking": false}}'

# Locale in which the AI explains the suggestion. Defaults to user system
# locale. Useful when you use small models that speak only English, for
# example.
locale = "en-US"

# Additional prompts to be included. (Yes, you can include role-playing
# prompts you pervert)
# - User's environment is Zsh running in Arch Linux.
# - You are a cute catgirl. Always use cute phrases and expressions to prove
#   your cuteness in the <note> section, including cat imitations like nya~,
#   にゃ~,　喵～.
additional_prompt = "User's environment is Zsh running in Arch Linux."
```

When `enabled` is not set:

- If an API key is configured (in `config.toml` or via `_PR_AI_API_KEY`), the
	module is enabled and uses that key.
- Otherwise, the module asks whether to use the built-in API key. The regular
	option applies to the current invocation only; the `(always)` options write
	`enabled = true` or `enabled = false` to the user configuration file.
- In non-interactive modes, or when no terminal is available, the module
	skips the request instead of asking.

The following environment variables are still supported and take precedence
over `config.toml`:

- `_PR_AI_API_KEY`: Your own API key
- `_PR_AI_URL`: Any OpenAI compatible URL (see `api_url` above)
- `_PR_AI_MODEL`: Model used
- `_PR_AI_EXTRA`: Extra Json field requests, e.g. `"temperature" = 0.5`
- `_PR_AI_EXTRA_BODY`: A Json object for the `extra_body` field, e.g. `{"chat_template_kwargs": {"enable_thinking": false}}`
- `_PR_AI_DISABLE`: Setting to any value disables AI integration
- `_PR_AI_LOCALE`: Locale in which the AI explains the suggestion
- `_PR_AI_ADDITIONAL_PROMPT`: Additional prompts to be included

Compile time variables: Default values for the respective variables above when not set

- `_DEF_PR_AI_API_KEY`
- `_DEF_PR_AI_URL`
- `_DEF_PR_AI_MODEL`

If default values were not provided, pay-respects' own values will be used
after you agree to use the built-in API key. Your request will be filtered to
avoid abuse usages. Request will then be forwarded to a LLM provider that will
not use your data for training. This service is provided free and is not
guaranteed to always work. Donations would be appreciated:

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

## Advanced Usages

For non-trivial suggestions, you can add more context as comments (for Bash and Zsh, interactive comments needs to be explicitly enabled):
```sh
rustup # how do I setup nightly toolchain?
```
Or just a comment:
```sh
# git cache credential for 3 hours
```
