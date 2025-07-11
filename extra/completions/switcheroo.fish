# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_switcheroo_global_optspecs
	string join \n v/verbose q/quiet h/help V/version
end

function __fish_switcheroo_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_switcheroo_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_switcheroo_using_subcommand
	set -l cmd (__fish_switcheroo_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c switcheroo -n "__fish_switcheroo_needs_command" -s v -l verbose -d 'Increase logging verbosity'
complete -c switcheroo -n "__fish_switcheroo_needs_command" -s q -l quiet -d 'Decrease logging verbosity'
complete -c switcheroo -n "__fish_switcheroo_needs_command" -s h -l help -d 'Print help'
complete -c switcheroo -n "__fish_switcheroo_needs_command" -s V -l version -d 'Print version'
complete -c switcheroo -n "__fish_switcheroo_needs_command" -f -a "execute" -d 'Executes a payload on a connected Switch'
complete -c switcheroo -n "__fish_switcheroo_needs_command" -f -a "device" -d 'Checks if a Switch is connected and booted to RCM mode'
complete -c switcheroo -n "__fish_switcheroo_needs_command" -f -a "list" -d 'Lists favorite binaries'
complete -c switcheroo -n "__fish_switcheroo_needs_command" -f -a "add" -d 'Add a favorite binary'
complete -c switcheroo -n "__fish_switcheroo_needs_command" -f -a "remove" -d 'Remove a favorite binary'
complete -c switcheroo -n "__fish_switcheroo_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c switcheroo -n "__fish_switcheroo_using_subcommand execute" -s f -l favorite -d 'Use a favorite payload instead' -r
complete -c switcheroo -n "__fish_switcheroo_using_subcommand execute" -s w -l wait -d 'Wait for device to be connected'
complete -c switcheroo -n "__fish_switcheroo_using_subcommand execute" -s v -l verbose -d 'Increase logging verbosity'
complete -c switcheroo -n "__fish_switcheroo_using_subcommand execute" -s q -l quiet -d 'Decrease logging verbosity'
complete -c switcheroo -n "__fish_switcheroo_using_subcommand execute" -s h -l help -d 'Print help'
complete -c switcheroo -n "__fish_switcheroo_using_subcommand device" -s w -l wait -d 'Wait for device to be connected'
complete -c switcheroo -n "__fish_switcheroo_using_subcommand device" -s v -l verbose -d 'Increase logging verbosity'
complete -c switcheroo -n "__fish_switcheroo_using_subcommand device" -s q -l quiet -d 'Decrease logging verbosity'
complete -c switcheroo -n "__fish_switcheroo_using_subcommand device" -s h -l help -d 'Print help'
complete -c switcheroo -n "__fish_switcheroo_using_subcommand list" -s v -l verbose -d 'Increase logging verbosity'
complete -c switcheroo -n "__fish_switcheroo_using_subcommand list" -s q -l quiet -d 'Decrease logging verbosity'
complete -c switcheroo -n "__fish_switcheroo_using_subcommand list" -s h -l help -d 'Print help'
complete -c switcheroo -n "__fish_switcheroo_using_subcommand add" -s v -l verbose -d 'Increase logging verbosity'
complete -c switcheroo -n "__fish_switcheroo_using_subcommand add" -s q -l quiet -d 'Decrease logging verbosity'
complete -c switcheroo -n "__fish_switcheroo_using_subcommand add" -s h -l help -d 'Print help'
complete -c switcheroo -n "__fish_switcheroo_using_subcommand remove" -s v -l verbose -d 'Increase logging verbosity'
complete -c switcheroo -n "__fish_switcheroo_using_subcommand remove" -s q -l quiet -d 'Decrease logging verbosity'
complete -c switcheroo -n "__fish_switcheroo_using_subcommand remove" -s h -l help -d 'Print help'
complete -c switcheroo -n "__fish_switcheroo_using_subcommand help; and not __fish_seen_subcommand_from execute device list add remove help" -f -a "execute" -d 'Executes a payload on a connected Switch'
complete -c switcheroo -n "__fish_switcheroo_using_subcommand help; and not __fish_seen_subcommand_from execute device list add remove help" -f -a "device" -d 'Checks if a Switch is connected and booted to RCM mode'
complete -c switcheroo -n "__fish_switcheroo_using_subcommand help; and not __fish_seen_subcommand_from execute device list add remove help" -f -a "list" -d 'Lists favorite binaries'
complete -c switcheroo -n "__fish_switcheroo_using_subcommand help; and not __fish_seen_subcommand_from execute device list add remove help" -f -a "add" -d 'Add a favorite binary'
complete -c switcheroo -n "__fish_switcheroo_using_subcommand help; and not __fish_seen_subcommand_from execute device list add remove help" -f -a "remove" -d 'Remove a favorite binary'
complete -c switcheroo -n "__fish_switcheroo_using_subcommand help; and not __fish_seen_subcommand_from execute device list add remove help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
