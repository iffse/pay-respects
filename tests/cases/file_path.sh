case="file path typo"
command="cd ./correc"
error="no such file or directory"
expect="cd ./correct"
pre='cd $TMP && mkdir -p correct'
