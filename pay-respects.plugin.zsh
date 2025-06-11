if (( $+commands[pay-respects] )); then
	eval "$(pay-respects zsh --alias)"
else
	echo "pay-respects is not in $PATH"
fi
