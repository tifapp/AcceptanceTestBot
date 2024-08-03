generate_uuid_v4() {
    local N B C='89ab'
    for (( N=0; N < 16; ++N )); do
        B=$(( RANDOM%256 ))
        case $N in
            6) printf '4%x' $(( B%16 )) ;; # UUID version 4
            8) printf '%c%x' ${C:$RANDOM%${#C}:1} $(( B%16 )) ;; # UUID variant
            3|5|7|9) printf '%02x-' $B ;;
            *) printf '%02x' $B ;;
        esac
    done
    echo
}

echo "✅ Previous state cleared..."

if [ -z "${!GITHUB_API_KEY}" ]; then
    echo "Uploading SSH key to github"

    # Define the SSH key path
    SSH_KEY_PATH="$HOME/.ssh/id_rsa"

    # Check if the SSH key already exists
    if [ -f "$SSH_KEY_PATH" ]; then
        echo "SSH key already exists at $SSH_KEY_PATH"
    else
        # Generate a new SSH key
        ssh-keygen -t rsa -b 4096 -f "$SSH_KEY_PATH" -N ""
        echo "SSH key generated at $SSH_KEY_PATH"
        echo "SSH Public Key"
        cat "$HOME/.ssh/id_rsa.pub"

        sslpub="$(cat ${HOME}/.ssh/id_rsa.pub |tail -1)"

        git_ssl_keyname="roswaal-$(generate_uuid_v4)"
        echo "$git_ssl_keyname"

        echo "Host github.com\nUser roswaaltifbot\nAddKeysToAgent yes\nUseKeychain yes\nIdentityFile ~/.ssh/id_rsa" > "$HOME/.ssh/id_rsa.pub"

        curl -L \
        -X POST \
        -H "Accept: application/vnd.github+json" \
        -H "Authorization: Bearer $GITHUB_API_KEY" \
        -H "X-GitHub-Api-Version: 2022-11-28" \
        https://api.github.com/user/keys \
        -d "{\"title\":\"$git_ssl_keyname\",\"key\":\"$sslpub\"}"
    fi
else
    echo "No GITHUB_API_KEY environment variable was specified, set this variable to generate a new SSH Key"
fi

if [ -d "FitnessProject" ]; then
    echo "🔵 Main frontend repo found, skipping cloning step..."
else
    echo "🔵 Cloning main frontend repo"
    git clone git@github.com:tifapp/FitnessProject.git
    if [ $? -ne 0 ]; then
        echo "🔴 Failed to clone the main frontend repo, exiting..."
        exit 1
    fi
    echo "✅ Successfully cloned main frontend repo"
fi
cd FitnessProject
echo "🔵 Pulling latest development branch"
git switch development
git pull origin development
if [ $? -ne 0 ]; then
    echo "🔴 Failed to pull the latest development branch, exiting..."
    exit 1
fi
echo "✅ Successfully pulled latest development branch"
cd ..
./setup_test_repo.sh
