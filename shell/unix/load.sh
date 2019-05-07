volta() {
    local EXIT_CODE
    local VOLTA_ROOT

    # Use the user's existing `VOLTA_HOME` environment value if set; otherwise,
    # use a default of `~/.volta`.
    VOLTA_ROOT="${VOLTA_HOME:-"$HOME/.volta"}"

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
