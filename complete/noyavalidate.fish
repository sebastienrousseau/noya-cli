complete -c noyavalidate -s s -l schema -d 'Validate each document against the JSON Schema 2020-12 at PATH (the schema may itself be YAML or JSON)' -r -F
complete -c noyavalidate -l fix -d 'Rewrite FILE in place via the CST formatter (lossless: byte-faithful for everything except normalised whitespace and line endings). With stdin input, the formatted bytes go to stdout'
complete -c noyavalidate -s q -l quiet -d 'Suppress success output'
complete -c noyavalidate -s h -l help -d 'Print help (see more with \'--help\')'
complete -c noyavalidate -s V -l version -d 'Print version'
