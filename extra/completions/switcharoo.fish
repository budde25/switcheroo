complete -c switcharoo -n "__fish_use_subcommand" -s h -l help -d 'Print help information'
complete -c switcharoo -n "__fish_use_subcommand" -s V -l version -d 'Print version information'
complete -c switcharoo -n "__fish_use_subcommand" -f -a "execute" -d 'Executes a payload'
complete -c switcharoo -n "__fish_use_subcommand" -f -a "device" -d 'Checks if a Switch in RCM mode is detected'
complete -c switcharoo -n "__fish_use_subcommand" -f -a "gui" -d 'Opens the GUI'
complete -c switcharoo -n "__fish_use_subcommand" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c switcharoo -n "__fish_seen_subcommand_from execute" -s w -l wait -d 'Wait for device to be connected'
complete -c switcharoo -n "__fish_seen_subcommand_from execute" -s h -l help -d 'Print help information'
complete -c switcharoo -n "__fish_seen_subcommand_from device" -s h -l help -d 'Print help information'
complete -c switcharoo -n "__fish_seen_subcommand_from gui" -s h -l help -d 'Print help information'
