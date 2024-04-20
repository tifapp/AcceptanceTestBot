# Roswaal

Our automated acceptance testing bot used at TiF.

## Features

- Creates acceptance test specifications in plain english.
  - Automatically opens a PR on the frontend repo containing the acceptance test.
- Monitors progress on all acceptance tests.
- Removes existing acceptance tests.
- Slackbot interface.

## Creating a New Test

New acceptance tests can be written in plain english like so:

```
New Test: Basic Leave Event through Exploration as Attendee

Step 1: Justin is signed in
Step 2: Justin wants to find the nearest event in Fresno
Step 3: After finding an event, Justin wants to join it
Step 4: After some pondering, Justin decides that he is not interested in the event and wants to leave
Step 5: Justin has now left the event

Requirement 1: Ensure Justin has signed into his account
Requirement 2: Search for events in Fresno, and go to the details for the nearest one
Requirement 3: Have Justin join the event
Requirement 4: Have Justin leave the event
Requirement 5: Ensure that Justin has left the event successfully
```

TODO: - Write the rest of this
