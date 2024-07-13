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
if [ ! -f .env ]; then
    echo "🔴 No .env file found. Make sure to create a .env file with the TEST_SSH_PRIVATE_KEY_HOME_PATH set to the path of your private ssh key."
    exit_with_failure_if_needed
fi
while IFS= read -r line || [ -n "$line" ]; do
    # Ignore lines that are comments or empty
    if [[ ! $line =~ ^#.* ]] && [[ -n $line ]]; then
      # Export the variable
      export $line
    fi
done < .env
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
