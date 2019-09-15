# Alternative source to load, uses specific bash extensions to the POSIX shell (BASH_SOURCE, <<<) hence the .bash extension
# Usage in ~/.bashrc: source ~/path/to/volta/load.bash
# Example: source ~/.volta/load.bash
# Or: [[ -s ~/.volta/load.bash ]] && source ~/.volta/load.bash

# TODO: test on windows, window WSL and osx. hahahaha.


# Example usage in bash:
# $ source .volta/load.bash # source it directly

# $ echo $VOLTA_HOME # was VOLTA_HOME defined?
# /home/mcarifio/.volta

# $ tr ':' '\n' <<< $PATH|grep volta # was the volta bin added to PATH?
# /home/mcarifio/.volta/bin

# $ volta --version # does the function run?
# 0.6.3
# volta which node # does $VOLTA_HOME/shim ... shim?
# /home/mcarifio/.volta/tools/image/node/12.10.0/6.10.3/bin/node
# $ unset VOLTA_HOME  # VOLTA_HOME _must_ be set
# $ volta --version
# -bash: VOLTA_HOME: Environment variable VOLTA_HOME undefined. Please export it.
# $ export VOLTA_HOME=$HOME/.volta # get it back
# $ node --version
# v12.10.0
 


# Don't pollute current environment with local functions _*
trap "unset -f _volta_setup _volta_error" EXIT

_volta_error() {
    echo $* 1>&2
    return 1  # not exit otherwise you exit the current session
}

# Expecting bash.
[[ -z "${BASH}" ]] && _volta_error "Expecting bash."

# Make _setup a function so you don't pollute the environment with working variables like `me` and `here`
#  and you don't have to keep track of them.
_volta_setup() {
    local me=$(realpath ${BASH_SOURCE})
    local here=${me%/*}
    export VOLTA_HOME=${VOLTA_HOME:-${here}}
    if [[ -d ${VOLTA_HOME} ]] ; then
        local bin=${VOLTA_HOME}/bin
        [[ -d ${bin} ]] || _volta_error "Expecting ${bin}, not found."
        # Add to PATH iff not already there. Next line is hack city.
        if ! tr ':' '\n' <<< ${PATH} | grep --silent --no-messages ${bin} ; then
            export PATH=${bin}:$PATH
        fi
    else
        _volta_error "Expecting ${VOLTA_HOME}."
    fi
}
_volta_setup

# Add anything local, up to you.
[[ -s ${VOLTA_HOME}/load-local.bash ]] && source ${VOLTA_HOME}/load-local.bash


volta() {
    local EXIT_CODE
    local VOLTA_ROOT

    # Since we could have sourced load.bash from any path and that sets up VOLTA_HOME, we can't generate a suitable default value.
    # Oh for the want of closures. And scoped functions.
    VOLTA_ROOT=${VOLTA_HOME:?"Environment variable VOLTA_HOME undefined. Please export it."}

	# Generate 32 bits of randomness, to avoid clashing with concurrent executions.
    export VOLTA_POSTSCRIPT="${VOLTA_ROOT}/tmp/volta_tmp_$(dd if=/dev/urandom count=1 2> /dev/null | cksum | cut -f1 -d" ").sh"

    # Forward the arguments to the Volta executable.
    VOLTA_SHELL=bash command "${VOLTA_ROOT}/volta" "$@"
    EXIT_CODE=$?

    # Call the post-invocation script if it is present, then delete it.
    # This allows the invocation to potentially modify the caller's environment (e.g., PATH).
    if [ -f "${VOLTA_POSTSCRIPT}" ]; then
        . "${VOLTA_POSTSCRIPT}"
        rm "${VOLTA_POSTSCRIPT}"
    fi

    unset VOLTA_POSTSCRIPT
    return $EXIT_CODE
}
