rm -drf FitnessProjectTest > /dev/null
echo "🔵 Creating test repo (this is used as an isolated environment for integration tests)"
mkdir -p FitnessProjectTest/roswaal
cd FitnessProjectTest
git init
git remote add origin git@github.com:roswaaltifbot/FitnessProjectTest.git
git branch -M main
git pull origin main
echo "✅ Successfully setup test repo"
