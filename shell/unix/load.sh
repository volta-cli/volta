jetson() {
    local EXIT_CODE
    local JETSON_ROOT

    # Use the user's existing `JETSON_HOME` environment value if set; otherwise,
    # use a default of `~/.jetson`.
    JETSON_ROOT="${JETSON_HOME:-"$HOME/.jetson"}"

	# Generate 32 bits of randomness, to avoid clashing with concurrent executions.
    export JETSON_POSTSCRIPT="${JETSON_ROOT}/tmp/jetson_tmp_$(dd if=/dev/urandom count=1 2> /dev/null | cksum | cut -f1 -d" ").sh"

    # Forward the arguments to the Jetson executable.
    JETSON_SHELL=bash command "${JETSON_ROOT}/jetson" "$@"
    EXIT_CODE=$?

    # Call the post-invocation script if it is present, then delete it.
    # This allows the invocation to potentially modify the caller's environment (e.g., PATH).
    if [ -f "${JETSON_POSTSCRIPT}" ]; then
        . "${JETSON_POSTSCRIPT}"
        rm "${JETSON_POSTSCRIPT}"
    fi

    unset JETSON_POSTSCRIPT
    return $EXIT_CODE
}
