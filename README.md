# SSH agent

This project is in progress.
It opens ssh connections and executes predefined commands asynchronously.
The idea is that it will have agents connecting to different hosts, gather and save the data in DB instance.
It's supposed to act like ansible, but asynchronously.

# TODO ideas

- maybe keep handle to ssh session within Process structure and move it between ssh_request, so that it doesn't need to reopen session every time.
- add db instance

# Usage

1. Prepare list of agents in hosts.json (see hosts.json.example for reference)

Single structure is as follows:

```
"username":"mylogin",
"private_key_path":"/home/mylogin/.ssh/id_rsa",
"ip":"127.0.0.1",
"port":"22",
"command":"ls -latr"
```

You can find example in hosts.json.example

2. Run program (make sure hosts.json or hosts.json.example is placed in current directory):

```
cargo run
```

