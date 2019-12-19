#!/bin/bash

cyan() {
    command printf '\033[1;36m'
    command printf "$1"
    command printf '\033[0m'
}

magenta() {
    command printf '\033[35m'
    command printf "$1"
    command printf '\033[0m'
}

info() {
  command printf '\033[1;33m[⚡ Volta CI ⚡]\033[0m %s\n' "$1" 1>&2
}

err() {
  command printf '\033[1;33m[⚡ Volta CI ⚡]\033[0m \033[1;31mError\033[0m: %s\n' "$1" 1>&2
}

top_commit() {
    local merge_commit_sha
    local top_commit_sha

    # Azure Pipelines stores a special merge commit for a PR with hash $(Build.SourceVersion)
    # https://docs.microsoft.com/en-us/azure/devops/pipelines/build/variables?view=azure-devops&tabs=yaml#build-variables
    # https://github.com/Microsoft/azure-pipelines-agent/issues/1980
    merge_commit_sha="$BUILD_SOURCEVERSION"

    # This merge commit points to the top commit of the PR as its second parent hash
    top_commit_sha=$(git show "${merge_commit_sha}" --pretty=%P | awk '{print $2;}')

    # Print the commit message for the top commit of the PR
    git log --format=%B -n 1 "${top_commit_sha}"
}

check_override() {
    local message
    local directive
    local result
    local pretty_directive

    message=$1
    directive=$2

    # Echo a non-empty string, which Azure Pipelines will treat as True, if and only if the override is set.
    # https://docs.microsoft.com/en-us/azure/devops/pipelines/process/expressions?view=azure-devops#type-casting
    result=$(echo "$message" | fgrep -q "$directive" && echo True)

    pretty_directive=$(cyan "$directive")
    info "Checking override $pretty_directive: ${result:-False}"

    echo $result
}

set_output_variable() {
    local varname
    local value
    local pretty_varname

    varname=$1
    value=$2

    pretty_varname=$(magenta "$varname")
    info "Setting job output variable: $pretty_varname=$value"

    echo "##vso[task.setvariable variable=$varname;isOutput=true]$value"
}

commit_message=$(top_commit)

info "Commit message:"

echo
echo "$commit_message" | sed 's/^/    > /'
echo

# FIXME: add an early check that this isn't from a fork repo, to give a better error message
docs=$(check_override "$commit_message" '[ci docs]')
if [[ "$SYSTEM_PULLREQUEST_ISFORK" == "True" && "$docs" == "True" ]]; then
    err 'Forks do not have permissions to publish docs.'
    exit 1
else
    set_output_variable docs $docs
fi
