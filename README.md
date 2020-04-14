# Silicon Valley Hacks Discord Bot
A simple bot for managing an online hackathon over Discord.
It can be deployed using Heroku entirely in the free-tier.
All data is persisted in a Redis instance to ensure maximum efficiency give the small amount of data.  

## Features
The features include reporting, team management, and mentor requests.
All commands are prefixed with `~` and can be accessed in any channel that the bot has read access.

### Command List
- Team Management
  - `~join <team number>`
  - `~leave <team_number>`
- Mentor Requests
  - `~mentor request <description>, [<link>]`
  - `~mentor list`
  - `~mentor complete <id>`
- Reporting
  - `~report <message>`
  - `~emergency [<message>]`
- Administrator
  - `~shutdown`
  
## Deployment
While being able to run entirely on the Heroku free-tier, you can also run it on your own server.
All configuration is done through environment variables.
The [`.env.example`](.env.example) can be used as reference.

### Heroku
Coming soon...

### Custom
1. Have a Redis instance running that is accessible by the project
1. Build the Rust project with `cargo build --release`
1. Copy it to your server
1. Configure your `.env` file using [`.env.example`](.env.example) as reference
