complete -c crabster -l port -d 'Port to listen on' -r
complete -c crabster -l bind -d 'Address to bind to' -r
complete -c crabster -l dir -d 'Directory to serve' -r -f -a "(__fish_complete_directories)"
complete -c crabster -l completions -d 'Generate shell completions' -r -f -a "bash\t''
elvish\t''
fish\t''
powershell\t''
zsh\t''"
complete -c crabster -l readonly -d 'Read-only mode (disable upload and delete)'
complete -c crabster -l hidden -d 'Show hidden files'
complete -c crabster -l no-delete -d 'Disable file deletion (uploads still allowed)'
complete -c crabster -l help -d 'Print help (see more with \'--help\')'
complete -c crabster -l version -d 'Print version'
