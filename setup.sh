rm -drf FitnessProjectTest > /dev/null
echo "✅ Previous state cleared..."
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
echo "🔵 Creating test repo (this is used as an isolated environment for integration tests)"
mkdir -p FitnessProjectTest/roswaal
touch FitnessProjectTest/roswaal/Locations.ts
echo "✅ Successfully setup test repo"
