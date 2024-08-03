echo "✅ Previous state cleared..."

# Define the SSH key path
SSH_KEY_PATH="$HOME/.ssh/id_rsa"

# Check if the SSH key already exists
if [ -f "$SSH_KEY_PATH" ]; then
    echo "SSH key already exists at $SSH_KEY_PATH"
else
    # Generate a new SSH key
    ssh-keygen -t rsa -b 4096 -f "$SSH_KEY_PATH" -N ""
    echo "SSH key generated at $SSH_KEY_PATH"
fi
echo "SSH Public Key"
cat "$HOME/.ssh/id_rsa.pub"

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
