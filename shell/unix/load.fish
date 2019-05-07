function volta
    # Use the user's existing `NOTION_HOME` environment value if set; otherwise,
    # use a default of `~/.volta`.
    if set -q NOTION_HOME;
        set NOTION_ROOT "$NOTION_HOME"
    else
        set NOTION_ROOT "$HOME/.volta"
    end

    # Generate 32 bits of randomness, to avoid clashing with concurrent executions.
    set -x NOTION_POSTSCRIPT "$NOTION_ROOT/tmp/volta_tmp_"(dd if=/dev/urandom count=1 2> /dev/null | cksum | cut -f1 -d" ")".fish"

    # Forward the arguments to the Volta executable.
    env NOTION_SHELL=fish command "$NOTION_ROOT/volta" $argv
    set EXIT_CODE $status

    # Call the post-invocation script if it is present, then delete it.
    # This allows the invocation to potentially modify the caller's environment (e.g., PATH).
    if test -f "$NOTION_POSTSCRIPT"
        source $NOTION_POSTSCRIPT
        rm "$NOTION_POSTSCRIPT"
    end

    set -e NOTION_POSTSCRIPT
    return $EXIT_CODE
end
