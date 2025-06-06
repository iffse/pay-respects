#
# Make it easier for zsh plugin managers to load pay-respects
#

if which pay-respects > /dev/null 2>&1; then
  eval "$(pay-respects zsh --alias)"
else
  echo "pay-respects is not in $PATH"
fi
