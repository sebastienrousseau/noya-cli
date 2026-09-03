complete -c noyafmt -l indent -d 'Indentation width in spaces' -r
complete -c noyafmt -l check -d 'Verify each FILE is formatted; print the list of files that need formatting and exit 1 if any do. Non-destructive. Suitable as a pre-commit / CI gate'
complete -c noyafmt -l write -d 'Rewrite each FILE in place. Default is to print the formatted source to stdout'
complete -c noyafmt -l stdin -d 'Read from stdin, write to stdout. Mutually exclusive with FILE arguments'
complete -c noyafmt -s h -l help -d 'Print help (see more with \'--help\')'
complete -c noyafmt -s V -l version -d 'Print version'
