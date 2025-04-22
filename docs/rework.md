A major architectural issue has arisen which is that it is quite difficult modify the parent shell.

As such direction has changed to opening the shells/windows that a user needs for a project.
That is the pipeline becomes 
input project -> project settings retrieval(db) -> windows open with settings

We will be moving away from manual config files to config being stored in sqlite. with config being done through some editor on a command being run like goto new <project-name> or got edit <project-name>

Thus we have settings stored as toml in an sqlite db

we will be using:
- clap
- toml/serde
- anyhow
- Possibly dialoguer 
- open
- rusqlite

db:
general config table
alias table
projects table

