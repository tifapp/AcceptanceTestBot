exit_with_failure () {
    echo "🔴 Failed to reset test repo, exiting..."
    exit 1
}

exit_with_failure_if_needed() {
    if [ $? -ne 0 ]; then
        exit_with_failure
    fi
}

echo "🔵 Resetting Test Repo"
cd FitnessProjectTest
git commit --allow-empty -n -m "No unborn branches ensured..." > /dev/null 2> /dev/null
git branch --show-current
git reset --hard HEAD
exit_with_failure_if_needed
git clean -fd
exit_with_failure_if_needed
git switch main
exit_with_failure_if_needed
cd ..
echo "✅ Successfully Reset Test Repo"
