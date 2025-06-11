if command -sq zoxide
	pay-respects fish --alias | source
else
	echo "pay-respects is not in $PATH"
end
