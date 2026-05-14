# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_crabster_global_optspecs
	string join \n port= bind= dir= readonly hidden no-delete daemon completions= auth= help version
end

function __fish_crabster_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_crabster_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_crabster_using_subcommand
	set -l cmd (__fish_crabster_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c crabster -n "__fish_crabster_needs_command" -l port -d 'Port to listen on' -r
complete -c crabster -n "__fish_crabster_needs_command" -l bind -d 'Address to bind to' -r
complete -c crabster -n "__fish_crabster_needs_command" -l dir -d 'Directory to serve' -r -f -a "(__fish_complete_directories)"
complete -c crabster -n "__fish_crabster_needs_command" -l completions -d 'Generate shell completions' -r -f -a "bash\t''
elvish\t''
fish\t''
powershell\t''
zsh\t''"
complete -c crabster -n "__fish_crabster_needs_command" -l auth -d 'Require a code (password) for access' -r
complete -c crabster -n "__fish_crabster_needs_command" -l readonly -d 'Read-only mode (disable upload and delete)'
complete -c crabster -n "__fish_crabster_needs_command" -l hidden -d 'Show hidden files'
complete -c crabster -n "__fish_crabster_needs_command" -l no-delete -d 'Disable file deletion (uploads still allowed)'
complete -c crabster -n "__fish_crabster_needs_command" -l daemon -d 'Run in background as a daemon'
complete -c crabster -n "__fish_crabster_needs_command" -l help -d 'Print help (see more with \'--help\')'
complete -c crabster -n "__fish_crabster_needs_command" -l version -d 'Print version'
complete -c crabster -n "__fish_crabster_needs_command" -f -a "ps" -d 'List running daemon instances'
complete -c crabster -n "__fish_crabster_needs_command" -f -a "stop" -d 'Stop a running daemon instance'
complete -c crabster -n "__fish_crabster_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c crabster -n "__fish_crabster_using_subcommand ps" -s h -l help -d 'Print help'
complete -c crabster -n "__fish_crabster_using_subcommand stop" -s h -l help -d 'Print help'
complete -c crabster -n "__fish_crabster_using_subcommand help; and not __fish_seen_subcommand_from ps stop help" -f -a "ps" -d 'List running daemon instances'
complete -c crabster -n "__fish_crabster_using_subcommand help; and not __fish_seen_subcommand_from ps stop help" -f -a "stop" -d 'Stop a running daemon instance'
complete -c crabster -n "__fish_crabster_using_subcommand help; and not __fish_seen_subcommand_from ps stop help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
