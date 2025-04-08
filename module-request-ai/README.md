# Request AI Module

Module for [pay-respects](https://codeberg.org/iff/pay-respects) to request AI for suggestions.

## Configurations

Configuration is done via environment variables:

- `_PR_AI_API_KEY`: Your own API key
- `_PR_AI_URL`: URL used. Any OpenAI compatible URL can be used, e.g.:
	- `https://api.openai.com/v1/chat/completions` (Note: OpenAI's ChatGPT is very slow)
	- `https://api.groq.com/openai/v1/chat/completions`
	- `http://localhost:11434/api/chat`: Ollama
- `_PR_AI_MODEL`: Model used
- `_PR_AI_DISABLE`: Setting to any value disables AI integration
- `_PR_AI_LOCALE`: Locale in which the AI explains the suggestion. Defaults to user system locale
- `_PR_AI_ADDITIONAL_PROMPT`: Additional prompts to be included. (Yes, you can include role-playing prompts you pervert)
	- `User's environment is Zsh running in Arch Linux.`
	- `You are a cute catgirl. Always use cute phrases and expressions to prove your cuteness in the note, including cat imitations like nya~, にゃ~,　喵～.`

Compile time variables: Default values for the respective variables above when not set

- `_DEF_PR_AI_API_KEY`
- `_DEF_PR_AI_URL`
- `_DEF_PR_AI_MODEL`

If default values were not provided, pay-respects' own values will be used. Your request will be filtered to avoid abuse usages. Request will then be forwarded to a LLM provider that will not use your data for training. This service is provided free and is not guaranteed to always work. Donations would be appreciated:

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

For non-trivial suggestions, you can add more context as comments (interactive comments needs to be explicitly enabled in Bash and Zsh):
```sh
pacman -S # how do I install Rust?
```
