case="regex: options"
command="rm correc -r"
error="no such file or directory"
expect="rm -r ./correct"
pre='cd $TMP && mkdir -p correct'
