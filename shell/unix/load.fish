function jetson
    # Use the user's existing `JETSON_HOME` environment value if set; otherwise,
    # use a default of `~/.jetson`.
    if set -q JETSON_HOME;
        set JETSON_ROOT "$JETSON_HOME"
    else
        set JETSON_ROOT "$HOME/.jetson"
    end

    # Generate 32 bits of randomness, to avoid clashing with concurrent executions.
    set -x JETSON_POSTSCRIPT "$JETSON_ROOT/tmp/jetson_tmp_"(dd if=/dev/urandom count=1 2> /dev/null | cksum | cut -f1 -d" ")".fish"

    # Forward the arguments to the Jetson executable.
    env JETSON_SHELL=fish command "$JETSON_ROOT/jetson" $argv
    set EXIT_CODE $status

    # Call the post-invocation script if it is present, then delete it.
    # This allows the invocation to potentially modify the caller's environment (e.g., PATH).
    if test -f "$JETSON_POSTSCRIPT"
        source $JETSON_POSTSCRIPT
        rm "$JETSON_POSTSCRIPT"
    end

    set -e JETSON_POSTSCRIPT
    return $EXIT_CODE
end
