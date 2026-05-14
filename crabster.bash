_crabster() {
    local i cur prev opts cmd
    COMPREPLY=()
    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
        cur="$2"
    else
        cur="${COMP_WORDS[COMP_CWORD]}"
    fi
    prev="$3"
    cmd=""
    opts=""

    for i in "${COMP_WORDS[@]:0:COMP_CWORD}"
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="crabster"
                ;;
            crabster,start)
                cmd="crabster__subcmd__start"
                ;;
            crabster,ps)
                cmd="crabster__subcmd__ps"
                ;;
            crabster,stop)
                cmd="crabster__subcmd__stop"
                ;;
            crabster,logs)
                cmd="crabster__subcmd__logs"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        crabster)
            opts="--port --bind --dir --readonly --hidden --no-delete --daemon --completions --auth --help --version start ps stop logs"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --port)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --bind)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --dir)
                    COMPREPLY=()
                    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
                        compopt -o plusdirs
                    fi
                    return 0
                    ;;
                --completions)
                    COMPREPLY=($(compgen -W "bash elvish fish powershell zsh" -- "${cur}"))
                    return 0
                    ;;
                --auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        crabster__subcmd__start)
            opts="--help"
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        crabster__subcmd__ps)
            opts="--help"
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        crabster__subcmd__stop)
            opts="--help"
            if [[ ${cur} == -* ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            # Dynamic completion for instance IDs from ~/.crabster/instances/
            local instances=$(ls -1 "$HOME/.crabster/instances/" 2>/dev/null | sed 's/\.json$//')
            COMPREPLY=( $(compgen -W "${instances}" -- "${cur}") )
            return 0
            ;;
        crabster__subcmd__logs)
            opts="-f --follow --help"
            if [[ ${cur} == -* ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            # Dynamic completion for instance IDs from ~/.crabster/instances/
            local instances=$(ls -1 "$HOME/.crabster/instances/" 2>/dev/null | sed 's/\.json$//')
            COMPREPLY=( $(compgen -W "${instances}" -- "${cur}") )
            return 0
            ;;
    esac
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _crabster -o nosort -o bashdefault -o default crabster
else
    complete -F _crabster -o bashdefault -o default crabster
fi
