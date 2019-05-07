function volta
    # Use the user's existing `VOLTA_HOME` environment value if set; otherwise,
    # use a default of `~/.volta`.
    if set -q VOLTA_HOME;
        set VOLTA_ROOT "$VOLTA_HOME"
    else
        set VOLTA_ROOT "$HOME/.volta"
    end

    # Generate 32 bits of randomness, to avoid clashing with concurrent executions.
    set -x VOLTA_POSTSCRIPT "$VOLTA_ROOT/tmp/volta_tmp_"(dd if=/dev/urandom count=1 2> /dev/null | cksum | cut -f1 -d" ")".fish"

    # Forward the arguments to the Volta executable.
    env VOLTA_SHELL=fish command "$VOLTA_ROOT/volta" $argv
    set EXIT_CODE $status

    # Call the post-invocation script if it is present, then delete it.
    # This allows the invocation to potentially modify the caller's environment (e.g., PATH).
    if test -f "$VOLTA_POSTSCRIPT"
        source $VOLTA_POSTSCRIPT
        rm "$VOLTA_POSTSCRIPT"
    end

    set -e VOLTA_POSTSCRIPT
    return $EXIT_CODE
end
